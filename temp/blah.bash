# Create a bash script to grep a json file for "command" and list all the unique values
# for the "command" key.  The script should take N JSON files as arguments and output the unique commands across all files.
# If a command is present in multiple files, it should only be listed once.

files=$@

for file in $files
do
    cat $file | grep -o '"action": "[^"]*' | cut -d'"' -f4
done | sort | uniq


