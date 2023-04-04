#!/usr/bin/python3
from lxml import etree
from statistics import mean, median
from tabulate import tabulate
import pprint
import sys

if len(sys.argv) < 2:
    print("Usage: convert.py <INPUT_FILE_NAME>", file=sys.stderr)
    sys.exit(-1)

input_file = sys.argv[1]

api_functions_ins = {}
api_functions_info_data = {}

tree = etree.parse(input_file)

for i in range(1):
#    root = tree.xpath("/root/" + f)
#    root = tree.xpath("//" + f)

# Look for all "kernel..." calls
    root = tree.xpath(".//*[starts-with(local-name(), 'kernel')]")
    for child in root:

        key = child.tag
        cpu_instructions = int(child.attrib["ins"])
        execute_cost = child.xpath(".//consume_cost_units")

        if cpu_instructions == 0 and "return" in child.attrib and child.attrib["return"] == "true":
            print("Skipping function which returned early: ", child.tag, "\n", file=sys.stderr)
            continue

#=====================================#
#    Add arguments to kernel calls    #
#=====================================#
        invoke_size = 0

        # handle kernel_inovke
        resolve = child.xpath("./self::kernel_invoke/resolve")
        if resolve:
            blueprint_name = resolve[0].attrib["arg0"].replace('"','')
            fcn_name = resolve[0].attrib["arg1"].replace('"','')
            invoke_size = resolve[0].attrib["arg2"]
            key += "::" + blueprint_name + "::" + fcn_name + "::" + invoke_size

        # handle node_id (from kernel_create_node, kernel_drop_node, kernel_get_node_visibility_origin, kernel_lock_substate)
        param = child.xpath("./@node_id")
        if param:
            if '"' in param[0]:
                key_param = param[0].partition('"')[0]
            elif '[' in param[0]:
                key_param = param[0].partition('[')[0]
            else:
                key_param = param[0]
            if key_param[-1] == '(': key_param = key_param[:-1]
            key += "::" + key_param

        # handle kernel_create_wasm_instance
        param = child.xpath("./self::kernel_create_wasm_instance/@package_address")
        if param:
            key += "::" + str(param[0])[:20] + "...]"
        param = child.xpath("./self::kernel_create_wasm_instance/create_instance/@arg0")
        if param:
            key += "::" + str(param[0])

        # handle kernel_lock_substate
        param = child.xpath("./self::kernel_lock_substate[@module_id | @offset]")
        if param:
            key_offset = param[0].attrib["offset"].partition('[')[0]
            if key_offset[-1] == '(': key_offset = key_offset[:-1]
            key += "::" + param[0].attrib["module_id"] + "::" + key_offset

        # handle kernel_allocate_node_id
        param = child.xpath("./self::kernel_allocate_node_id/@node_type")
        if param:
            key += "::" + param[0]

        # correcting parenthesis
        c1 = key.count('(')
        c2 = key.count(')')
        key += ')'*(c1-c2)

#===========================================================================================================#
#    Subtract nested kernel calls with separate consumed costs CPU instruction from current kernel call     #
#===========================================================================================================#
        kernel_nested_calls = child.xpath("./*[starts-with(local-name(), 'kernel') and .//consume_multiplied_execution]")
        #print("kernel nested calls count: ", len(kernel_nested_calls))
        for i in kernel_nested_calls:
            #print(" call: ", tree.getpath(i))
            cpu_instructions -= int(i.attrib["ins"])

#=============================================================================================================================#
#    Subtract nested kernel calls under execute tag with separate consumed costs CPU instruction from current kernel call     #
#=============================================================================================================================#
        kernel_nested_calls = child.xpath("./execute/*[starts-with(local-name(), 'kernel') and .//consume_multiplied_execution]")
        #print("kernel nested calls count: ", len(kernel_nested_calls))
        for i in kernel_nested_calls:
            #print(" call: ", tree.getpath(i))
            cpu_instructions -= int(i.attrib["ins"])

#=====================================#
#    Extract sum of all cost units    #
#=====================================#
        cost = 0
        for i in execute_cost:
            cost = cost + int(i.attrib["units"])
        info_data = "(cost: " + str(cost) + ", ins: " + child.attrib["ins"]  + ", size: " + str(invoke_size) + ")"
        api_functions_info_data[key] = info_data

#        if not child.tag in api_functions:
#            continue

        if api_functions_ins.get(key):
            api_functions_ins[key][1].append(cpu_instructions)
        else:
            api_functions_ins[key] = (info_data,[cpu_instructions]) # remove info_data?

output = {}
output_tab = []

for i in api_functions_ins.keys():
    output_tab.append([i, len(api_functions_ins[i][1]), min(api_functions_ins[i][1]), max(api_functions_ins[i][1]), round(mean(api_functions_ins[i][1])), round(median(api_functions_ins[i][1])) ])

#sorted(output_tab, key=lambda x: x[0])
output_tab.sort()
for idx, item in enumerate(output_tab):
    item.insert(0,idx + 1)

output_tab.insert(0,["No.", "API function with params", "calls count", "min instr.", "max instr.", "mean", "median"])

print(tabulate(output_tab, headers="firstrow"))

