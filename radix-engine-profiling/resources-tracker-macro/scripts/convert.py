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
    if str(os.path.splitext(input_file)[1]).lower() != ".xml":
        print("Skipping file: ", input_file, " (extenstion not xml)", file=sys.stderr)
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
        #print(tree.getpath(child))

        if cpu_instructions == 0 and "return" in child.attrib and child.attrib["return"] == "true":
            #print("Skipping function which returned early: ", child.tag, "\n", file=sys.stderr)
            continue
        elif cpu_instructions == 0:
            parent_return = child.xpath(".//*[ancestor-or-self::*[@return='true']]")
            if parent_return:
                #print("Skipping function which parent returned early: ", child.tag, "\n", file=sys.stderr)
                continue

#=====================================#
#    Add arguments to kernel calls    #
#=====================================#

        # handle kernel_inovke
        invoke_size = 0
        resolve_size = child.xpath("./self::kernel_invoke/before_invoke")
        if resolve_size:
            invoke_size = resolve_size[0].attrib["arg0"]
        resolve = child.xpath("./self::kernel_invoke/invoke")
        if resolve:
            native = "no_native"
            if resolve[0].attrib["arg0"] == "true":
                native = "native"
            pkg = resolve[0].attrib["arg1"].replace('"','')
            fcn_name = resolve[0].attrib["export_name"]
            if fcn_name in kernel_invoke_divide_by_size:
                key += "::" + native + "::" + pkg + "::" + fcn_name + "::" + invoke_size
            else:
                key += "::" + native + "::" + pkg + "::" + fcn_name

        # handle kernel_create_node
        param = child.xpath("./self::kernel_create_node/@arg0")
        if param:
            key += "::" + param[0]

        # handle kernel_drop_node
        param = child.xpath("./self::kernel_drop_node/@arg0")
        if param:
            key += "::" + param[0]

        # handle kernel_get_node_info
        param = child.xpath("./self::kernel_get_node_info/@arg0")
        if param:
            key += "::" + param[0]

        # handle kernel_open_substate
        param = child.xpath("./self::kernel_open_substate[@module_id | @arg0 | @arg2]")
        if param:
            module_id = param[0].attrib["module_id"]
            node_id = param[0].attrib["arg0"]
            key += "::" + module_id + "::" + node_id

        # correcting parenthesis
        c1 = key.count('(')
        c2 = key.count(')')
        key += ')'*(c1-c2)

#======================================================================================#
#    Subtract child nested calls, looking up to 5 childs, for example:                 #
#     kernel_invoke/kernel_invoke                                                      #
#     kernel_invoke/invoke_export/kernel_invoke                                        #
#     kernel_invoke/other_tag1/other_tag2/kernel_invoke                                #
#     kernel_invoke/t1/t2/t3/kernel_invoke                                             #
#     kernel_invoke/t1/t2/t3/t4/kernel_invoke                                          #
#======================================================================================#
        nested_call_suffix = "/*[local-name() = '" + child.tag + "']"
        nested_call_prefix = "."
        nested_call_middle = ""
        for i in range(5):
            nested_call_xpath = nested_call_prefix + nested_call_middle + nested_call_suffix
            kernel_nested_calls = child.xpath(nested_call_xpath)
            #if len(kernel_nested_calls) > 0:
                #print("  direct child nested calls count: ", len(kernel_nested_calls), nested_call_xpath, cpu_instructions)
            for i in kernel_nested_calls:
                cpu_instructions -= int(i.attrib["ins"])
                #print("   call: ", tree.getpath(i), cpu_instructions)
            nested_call_middle += "/*[not(local-name() = '" + child.tag + "')]"

#        if not child.tag in api_functions:
#            continue

        if api_functions_ins.get(key):
            api_functions_ins[key].append(cpu_instructions)
        else:
            api_functions_ins[key] = [cpu_instructions]

# end of xml parsing loop


# prepare output

output = {}
output_tab = []

if detailed_output:
    for i in api_functions_ins.keys():
        output_tab.append([i, len(api_functions_ins[i]), min(api_functions_ins[i]), max(api_functions_ins[i]), round(mean(api_functions_ins[i])), round(median(api_functions_ins[i])) ])
else:
    for i in api_functions_ins.keys():
        if i.split(':')[0] in use_max_instead_of_median:
            output_tab.append([i, max(api_functions_ins[i]), "max" ])
        else:
            output_tab.append([i, round(median(api_functions_ins[i])), "median" ])

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
    f.close()

    f = open("/tmp/native_function_base_costs.csv", "w")
    # skip header row
    for row in output_tab[1:]:
        token = "kernel_invoke::native::"
        if type(row[1]) == str and row[1].find(token) == 0:
            package_address_and_function = str(row[1][len(token):])
            if package_address_and_function.count("::") == 1:
                package_address_and_function = package_address_and_function.replace("::",",")
                f.write( package_address_and_function + "," + str(row[2]))
                f.write("\n")
            #else: function with additional size parameter
    f.close()
