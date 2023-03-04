#!/usr/bin/bash

our_version=$(cargo readme -V | perl -ne 'print $1 while  /v([\d.]+)/g')
last_version=$(cargo search --color=never cargo-readme | perl -ne 'print $1 while /^cargo-readme = "([\d.]+)"/g')

if [ "$our_version" == "$last_version" ]; then
    echo Version $our_version is of cargo-readme is installed and up to date.
else
    echo "Install cargo-readme"
    cargo install cargo-readme
fi
