#!/bin/sh

localedir="${LOCALEDIR:-/usr/share/locale}"
build_alias="${BUILD_ALIAS:-}"
host_alias="${HOST_ALIAS:-}"
print_localedir_bin="${PRINT_LOCALEDIR_BIN:-./print-localedir}"

binlocaledir="$("${print_localedir_bin}" 2> /dev/null | sed -n '/./{p;q;}')"

if test "${localedir}" = "${binlocaledir}"; then
    echo "Makefile and binary agree on localedir \`${localedir}'. Good."
    exit 0
else
    echo "Makefile and binary disagree on localedir. Bad."
    echo "  - Makefile says \`${localedir}'."
    echo "  - binary   says \`${binlocaledir}'."

    if test "${build_alias}" != "${host_alias}"; then
	echo "However, you are cross-compiling, so this does not necessarily"
	echo "have consequences."
	exit 0
    else
	echo "Error: Could not determine binary localedir."
	exit 1
    fi
fi
