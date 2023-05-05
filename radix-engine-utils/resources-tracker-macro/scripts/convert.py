#!/usr/bin/python3
#
# Required python packages: lxml, tabulate. To install run command: pip3 install lxml tabulate
#

from lxml import etree
from statistics import mean, median
from tabulate import tabulate
import pprint
import sys
import os

if len(sys.argv) < 2:
    print("Usage: convert.py <INPUT_FOLDER> <DETAILED_OUTPUT_TABLE>\n\nwhere: <DETAILED_OUTPUT_TABLE>: 1 or 0", file=sys.stderr)
    sys.exit(-1)

detailed_output = 0
if len(sys.argv) == 3:
    detailed_output = sys.argv[2]

input_folder = sys.argv[1]
if not os.path.exists(os.path.dirname(input_folder)):
    print("Path: ", input_folder, " does not exists.", file=sys.stderr)
    sys.exit(-1)

api_functions_ins = {}
api_functions_info_data = {}

kernel_invoke_divide_by_size = [ "publish_native", "publish_wasm_advanced" ]
use_max_instead_of_median = ["kernel_create_wasm_instance"] # due to use of caching


for path in os.listdir(input_folder):
    input_file = os.path.join(input_folder, path)
    if not os.path.isfile(input_file):
        continue

    # parse xml file
    tree = 0
    try:
        tree = etree.parse(input_file)
    except (IOError, etree.ParseError):
        print("Cannot parse file: ", input_file, " - skipping", file=sys.stderr)
        continue

    #print("Parsing file: ", input_file)

    # Look for all "kernel..." calls
    root = tree.xpath(".//*[starts-with(local-name(), 'kernel')]")
    for child in root:

        key = child.tag
        cpu_instructions = int(child.attrib["ins"])
        execute_cost = child.xpath(".//consume_cost_units")

        if cpu_instructions == 0 and "return" in child.attrib and child.attrib["return"] == "true":
            #print("Skipping function which returned early: ", child.tag, "\n", file=sys.stderr)
            continue

#=====================================#
#    Add arguments to kernel calls    #
#=====================================#

        # handle kernel_inovke
        invoke_size = 0
        resolve = child.xpath("./self::kernel_invoke/before_invoke")
        if resolve:
            blueprint_name = resolve[0].attrib["arg1"].replace('"','')
            fcn_name = resolve[0].attrib["arg2"].replace('"','')
            receiver = resolve[0].attrib["arg3"].replace('"','') # not used
            if receiver == "None":
                receiver = ""
            else:
                receiver = "::" + receiver
            if fcn_name in kernel_invoke_divide_by_size:
                invoke_size = resolve[0].attrib["input_size"]
                key += "::" + blueprint_name + "::" + fcn_name + "::" + invoke_size
            else:
                key += "::" + blueprint_name + "::" + fcn_name

        # handle kernel_create_node
        param = child.xpath("./self::kernel_create_node/@arg0")
        if param:
            key += "::" + param[0]

        # handle kernel_drop_node
        param = child.xpath("./self::kernel_drop_node/@arg0")
        if param:
            key += "::" + param[0]

        # handle kernel_get_node_visibility
        param = child.xpath("./self::kernel_get_node_visibility/@arg0")
        if param:
            key += "::" + param[0]

        # handle kernel_lock_substate
        param = child.xpath("./self::kernel_lock_substate[@module_id | @arg0 | @arg2]")
        if param:
            module_id = param[0].attrib["module_id"]
            node_id = param[0].attrib["arg0"]
            key += "::" + module_id + "::" + node_id

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

# end of xml parsing loop


# prepare output

output = {}
output_tab = []

if detailed_output:
    for i in api_functions_ins.keys():
        output_tab.append([i, len(api_functions_ins[i][1]), min(api_functions_ins[i][1]), max(api_functions_ins[i][1]), round(mean(api_functions_ins[i][1])), round(median(api_functions_ins[i][1])) ])
else:
    for i in api_functions_ins.keys():
        if i.split(':')[0] in use_max_instead_of_median:
            output_tab.append([i, max(api_functions_ins[i][1]), "max" ])
        else:
            output_tab.append([i, round(median(api_functions_ins[i][1])), "median" ])

#sorted(output_tab, key=lambda x: x[0])
output_tab.sort()
for idx, item in enumerate(output_tab):
    item.insert(0,idx + 1)

if detailed_output:
    output_tab.insert(0,["No.", "API function with params", "calls count", "min instr.", "max instr.", "mean", "median"])
    print(tabulate(output_tab, headers="firstrow"))
else:
    min_ins = sys.maxsize
    if len(output_tab) > 0 and len(output_tab[0]) > 2:
        min_ins = output_tab[0][2]
    for row in output_tab:
        if row[2] < min_ins and row[2] > 0:
            min_ins = row[2]
    mul_div = 16
    for row in output_tab:
        coeff = round(mul_div * row[2] / min_ins)
        row.append(coeff)
        row.append( row[2] - coeff * min_ins / mul_div )
    output_tab.insert(0,["No.", "API function with params", "instructions", "calculation", "function cost (F)", "error"])
    print(tabulate(output_tab, headers="firstrow"))
    print("\nCost function coeff: ", min_ins, "\n")
    print("Cost calculation = ( F *", min_ins, ") /", mul_div, "\n")

    f = open("/tmp/_out_table.csv", "w")
    for row in output_tab:
        for col in row:
            if type(col) == str and col.count(';') > 0:
                f.write("\"")
                f.write(col)
                f.write("\";")
            else:
                f.write(str(col))
                f.write(";")
        f.write("\n")

