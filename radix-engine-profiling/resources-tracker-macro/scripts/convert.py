#!/usr/bin/python3
#
# Required python packages: lxml, tabulate, numpy, scikit-learn, statsmodels. 
# To install run command: pip3 install lxml tabulate numpy scikit-learn statsmodels
#

from lxml import etree
from statistics import mean, median
from tabulate import tabulate
from sklearn.linear_model import LinearRegression
import pprint
import sys
import os
import numpy as np


if len(sys.argv) < 2:
    print("Usage: convert.py <INPUT_FOLDER_WITH_XML_FILES>\n\nResults are generated in files:", file=sys.stderr)
    print(" /tmp/_out_table.txt\n /tmp/_out_table_detailed.txt\n /tmp/_out_linear_regression_coeff.txt\n /tmp/native_function_base_costs.csv", file=sys.stderr)
    sys.exit(-1)

input_folder = sys.argv[1]
if not os.path.exists(os.path.dirname(input_folder)):
    print("Path: ", input_folder, " does not exists.", file=sys.stderr)
    sys.exit(-1)

api_functions_ins = {}
api_functions_info_data = {}

# Define functions which should be used for linear regression based on defined parameter (size, count, etc.)
kernel_invoke_divide_by_param = [ "publish_native", "publish_wasm_advanced", "kernel_drain_substates" ]

use_max_instead_of_median = ["kernel_create_wasm_instance"] # due to use of caching

file_list_cnt = 1
file_list = os.listdir(input_folder)
overflowed_cnt = 0
values_cnt = 0

for path in file_list:
    input_file = os.path.join(input_folder, path)

    print("Parsing file ", file_list_cnt, "/", len(file_list), ": ", input_file)
    file_list_cnt += 1

    if not os.path.isfile(input_file):
        continue
    if str(os.path.splitext(input_file)[1]).lower() != ".xml":
        print("Skipping file: ", input_file, " (extension not xml)", file=sys.stderr)
        continue

    # parse xml file
    tree = 0
    try:
        tree = etree.parse(input_file)
    except (IOError, etree.ParseError):
        print("Cannot parse file: ", input_file, " - skipping", file=sys.stderr)
        continue

    # Look for all "kernel..." calls
    try:
        root = tree.xpath(".//*[starts-with(local-name(), 'kernel') or starts-with(local-name(), 'before_invoke') or starts-with(local-name(), 'after_invoke')]")
    except:
        print("Cannot apply xpath expression", file=sys.stderr)
        continue
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
            if fcn_name in kernel_invoke_divide_by_param:
                key += "::" + native + "::" + pkg + "::" + fcn_name + "::" + invoke_size
            else:
                key += "::" + native + "::" + pkg + "::" + fcn_name

        # handle kernel_drain_substates
        param = child.xpath("./self::kernel_drain_substates/@limit")
        if param:
            key += "::" + param[0]

        # handle kernel_read_substate
        resolve_heap_or_track = child.xpath("./self::kernel_read_substate/on_read_substate")
        if resolve_heap_or_track:
            read_from_heap = resolve_heap_or_track[0].attrib["arg0"]
            if read_from_heap == "true":
                key += "::heap"
            else:
                key += "::store"

        # correcting parenthesis
        c1 = key.count('(')
        c2 = key.count(')')
        key += ')'*(c1-c2)

#======================================================================================#
#    Subtract child nested calls, looking up to 5 children, for example:                 #
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

        # discard values overflowed after calibration apply
        if cpu_instructions > 0xff00000000000000:
            overflowed_cnt += 1
            continue

        values_cnt += 1
        if api_functions_ins.get(key):
            api_functions_ins[key].append(cpu_instructions)
        else:
            api_functions_ins[key] = [cpu_instructions]

# end of xml parsing loop

print(" Overflowed values count: ", overflowed_cnt, "/", values_cnt)

# prepare output

output = {}
output_tab = []
output_tab_detailed = []

for i in api_functions_ins.keys():
    output_tab_detailed.append([i, len(api_functions_ins[i]), min(api_functions_ins[i]), max(api_functions_ins[i]), round(mean(api_functions_ins[i])), round(median(api_functions_ins[i])) ])

for i in api_functions_ins.keys():
    if i.split(':')[0] in use_max_instead_of_median:
        output_tab.append([i, max(api_functions_ins[i]), "max" ])
    else:
        output_tab.append([i, round(median(api_functions_ins[i])), "median" ])

output_tab_detailed.sort()
for idx, item in enumerate(output_tab_detailed):
    item.insert(0,idx + 1)

output_tab.sort()
for idx, item in enumerate(output_tab):
    item.insert(0,idx + 1)

# calculate linear approximation coefficients for selected functions and write to file
f = open("/tmp/_out_linear_regression_coeff.txt", "w")
for s in kernel_invoke_divide_by_param:
    values_size = []
    values_instructions = []
    for row in output_tab:
        if row[1].find("::" + s + "::") != -1 or row[1].find(s + "::") == 0:
            # extracting size from 2nd table column
            last_token_idx = row[1].rfind("::") + 2
            size = row[1][last_token_idx:]
            values_size.append(int(size))
            values_instructions.append(int(row[2]))
    if len(values_size) > 0:
        x = np.array(values_size).reshape((-1, 1))
        y = np.array(values_instructions)
        model = LinearRegression().fit(x, y)
        r_sq = model.score(x, y)
        intercept = model.intercept_
        slope = model.coef_[0]
        out_str = s + ": " + "instructions(param) = " + str(slope) + " * param + " + str(intercept) + "\n"
        f.write(out_str)
    else:
        print("Linear regression param not found for '",s,"' function.")
f.close()



# Write detailed output table
f = open("/tmp/_out_table_detailed.txt", "w")
output_tab_detailed.insert(0,["No.", "API function with params", "calls count", "min instr.", "max instr.", "mean", "median"])
f.write(tabulate(output_tab_detailed, headers="firstrow"))
f.write("\n")
f.close()

# Write results output table
f = open("/tmp/_out_table.txt", "w")
output_tab.insert(0,["No.", "API function with params", "instructions", "calculation"])
f.write(tabulate(output_tab, headers="firstrow"))
f.close()

# Write results output table as csv file
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

# Write results for native function base costs csv file
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
