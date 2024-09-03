#!/usr/bin/env bash
#### Script to add rust tags to vim, so that you can move back and forth from within vim.
#### 
#### Script will check all the subdirs with a Cargo.toml file within, and add rust tags.
if ! command -v rusty-tags &> /dev/null
then
    echo "Command 'rusty-tags' could not be found."
		echo ""
		echo "1. Install 'ctags' using apt, yum, dnf, or brew."
		echo "2. Install 'rusty-tags' with: 'cargo install rusty-tags'"
		echo "Setup editor: https://docs.rs/crate/rusty-tags/1.0.1"
		echo 
    exit
fi

if [ $# -eq 0 ]; then
	  echo
    echo "Error: need an argument of \"vi\" or \"emacs\"!" 
    echo 
    exit 1
fi

#SCRIPT_DIR= scripts directory no matter where it was called from
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
for tag_this in $(find .|grep Cargo.toml); do
	tag_dir=${tag_this/\/Cargo.toml/}
	tag_dir=$SCRIPT_DIR/${tag_dir/\.\//}
	cd $tag_dir
	rusty-tags vi
done
