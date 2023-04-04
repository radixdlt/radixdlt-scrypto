#!/usr/bin/python3
from lxml import etree
from statistics import mean, median
from tabulate import tabulate
import pprint
import sys


input_file = "./out.xml"
#input_file = "./test1.xml"


#print("started")

api_functions1 = ("kernel_drop_node", "kernel_allocate_node_id", "kernel_create_node", "kernel_get_node_visibility_origin", "kernel_get_current_actor", 
"kernel_read_bucket", "kernel_read_proof", "kernel_lock_substate", "kernel_get_lock_info", "kernel_drop_lock", "kernel_read_substate", "kernel_get_substate_ref",
"kernel_get_substate_ref_mut", "kernel_create_wasm_instance", "kernel_invoke" )

api_functions = ("kernel_invoke","kernel_create_node","kernel_lock_substate")

api_functions_ins = {}
api_functions_info_data = {}



tree = etree.parse(input_file)

#for f in api_functions:
for i in range(1):
#    root = tree.xpath("/root/" + f)
#    root = tree.xpath("//" + f)

# Look for all "kernel..." calls
    root = tree.xpath(".//*[starts-with(local-name(), 'kernel')]")
    for child in root:
        #print("\n",tree.getpath(child), " ", child.attrib["ins"])

        key = child.tag
        cpu_instructions = int(child.attrib["ins"])
        execute_cost = child.xpath(".//consume_cost_units")

        if cpu_instructions == 0 and child.attrib["return"] == "true":
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
            #print("adding resolve:   ", blueprint_name, fcn_name)
            key += "::" + blueprint_name + "::" + fcn_name + "::" + invoke_size

        # handle node_id (from kernel_create_node, kernel_drop_node, kernel_get_node_visibility_origin)
        param = child.xpath("./@node_id")
        if param:
            if '"' in param[0]:
                key_param = param[0].partition('"')[0]
            elif '[' in param[0]:
                key_param = param[0].partition('[')[0]
            else:
                key_param = param[0]
            key += "::" + key_param

        # handle kernel_create_wasm_instance
        param = child.xpath("./self::kernel_create_wasm_instance/@package_address")
        if param:
            key += "::" + str(param[0])
        param = child.xpath("./self::kernel_create_wasm_instance/create_instance/@arg0")
        if param:
            key += "::" + str(param[0])

        # handle kernel_lock_substate
        param = child.xpath("./self::kernel_lock_substate[@arg0 | @module_id | @offset]")
        if param:
            key_object = param[0].attrib["arg0"].partition('(')[0] #[:16]
            key_offset = param[0].attrib["offset"].partition('[')[0]
            key += "::" + key_object + "::" + param[0].attrib["module_id"] + "::" + key_offset

        # handle kernel_allocate_node_id
        param = child.xpath("./self::kernel_allocate_node_id/@node_type")
        if param:
            key += "::" + param[0]

        # handle kernel_read_substate
        param = child.xpath("./self::kernel_read_substate/on_read_substate/@size]")
        if param:
            key += "::" + param[0]

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

    #for child2 in child:
    #    print("  ", child2.tag, " ", child2.attrib)


#print("\n---\n")

output = {}
output_tab = []

for i in api_functions_ins.keys():
    output_tab.append([i, len(api_functions_ins[i][1]), min(api_functions_ins[i][1]), max(api_functions_ins[i][1]), round(mean(api_functions_ins[i][1])), round(median(api_functions_ins[i][1])) ])
#    output[i] = [min(api_functions_ins[i][1]), max(api_functions_ins[i][1]), round(mean(api_functions_ins[i][1]))]
#    print(i,"\t", min(api_functions_ins[i]), max(api_functions_ins[i]), round(mean(api_functions_ins[i]),2))
#print(len(r))

#sorted(output_tab, key=lambda x: x[0])
output_tab.sort()
output_tab.insert(0,["API call with params", "calls count", "max instr.", "max instr.", "mean", "median"])

#pprint.pprint(api_functions_ins)
print(tabulate(output_tab, headers="firstrow"))
#pprint.pprint(output)

