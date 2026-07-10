#!/bin/bash

set -e

cd $(dirname $0)

# end_of_line
[ -d end_of_line ] || mkdir -p end_of_line
pushd end_of_line
    [ -d crlf ] || mkdir -p crlf
    pushd crlf
        echo -ne "root = true\n[*.target]\nend_of_line = crlf\n" > .editorconfig
        echo -ne "a\r\nb\r\nc\r\n" > no_error.target
        echo -ne "a\nb\nc\n" > error_lf.target
        echo -ne "a\rb\rc\r" > error_cr.target
    popd
    [ -d lf ] || mkdir -p lf
    pushd lf
        echo -ne "root = true\n[*.target]\nend_of_line = lf\n" > .editorconfig
        echo -ne "a\nb\nc\n" > no_error.target
        echo -ne "a\r\nb\r\nc\r\n" > error_crlf.target
        echo -ne "a\rb\rc\r" > error_cr.target
    popd
    [ -d cr ] || mkdir -p cr
    pushd cr
        echo -ne "root = true\n[*.target]\nend_of_line = cr\n" > .editorconfig
        echo -ne "a\rb\rc\r" > no_error.target
        echo -ne "a\r\nb\r\nc\r\n" > error_crlf.target
        echo -ne "a\nb\nc\n" > error_lf.target
    popd
popd

# indent_style
[ -d indent_style ] || mkdir -p indent_style
pushd indent_style
    [ -d space ] || mkdir -p space
    pushd space
        echo -ne "root = true\n[*.target]\nindent_style = space\n" > .editorconfig
        echo -ne "a\n  b\nc\n" > no_error.target
        echo -ne "a\nb\nc\n" > unindented.target
        echo -ne "a\n\t\tb\nc\n" > error_tab.target
        echo -ne "a\n \tb\nc\n" > error_mixed_tab.target
    popd
    [ -d tab ] || mkdir -p tab
    pushd tab
        echo -ne "root = true\n[*.target]\nindent_style = tab\n" > .editorconfig
        echo -ne "a\n\t\tb\nc\n" > no_error.target
        echo -ne "a\nb\nc\n" > unindented.target
        echo -ne "a\n  b\nc\n" > error_space.target
        echo -ne "a\n\t b\nc\n" > error_mixed_space.target
    popd
    [ -d case_insensitive ] || mkdir -p case_insensitive
    pushd case_insensitive
        [ -d space ] || mkdir -p space
        pushd space
            echo -ne "root = true\n[*.target]\nindent_style = SPACE\n" > .editorconfig
            echo -ne "a\n  b\nc\n" > no_error.target
        popd
        [ -d tab ] || mkdir -p tab
        pushd tab
            echo -ne "root = true\n[*.target]\nindent_style = TAB\n" > .editorconfig
            echo -ne "a\n\t\tb\nc\n" > no_error.target
        popd
    popd
    [ -d unset ] || mkdir -p unset
    pushd unset
        echo -ne "root = true\n[*.target]\nindent_style = tab\n" > .editorconfig
        [ -d nested ] || mkdir -p nested
        pushd nested
            echo -ne "[*.target]\nindent_style = unset\n" > .editorconfig
            echo -ne "  a\n\tb\nc\n" > no_error.target
        popd
    popd
popd

# indent_size
[ -d indent_size ] || mkdir -p indent_size
pushd indent_size
    [ -d 2 ] || mkdir -p 2
    pushd 2
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = 2\n" > .editorconfig
        echo -ne "a\n  b\nc\n" > no_error.target
        echo -ne "a\n   b\nc\n" > error_3.target
    popd
    [ -d 4 ] || mkdir -p 4
    pushd 4
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = 4\n" > .editorconfig
        echo -ne "a\n    b\nc\n" > no_error.target
        echo -ne "a\n  b\nc\n" > error_2.target
    popd
    [ -d case_insensitive ] || mkdir -p case_insensitive
    pushd case_insensitive
        echo -ne "root = true\n[*.target]\nindent_style = SPACE\nindent_size = 2\n" > .editorconfig
        echo -ne "a\n   b\nc\n" > error_3.target
    popd
    [ -d one ] || mkdir -p one
    pushd one
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = 1\n" > .editorconfig
        echo -ne "a\n b\n       c\n" > no_error.target
    popd
    [ -d unset ] || mkdir -p unset
    pushd unset
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = 2\n[unset.target]\nindent_size = unset\n" > .editorconfig
        echo -ne "a\n   b\nc\n" > unset.target
    popd
    [ -d tab_without_tab_width ] || mkdir -p tab_without_tab_width
    pushd tab_without_tab_width
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = TAB\n" > .editorconfig
        echo -ne "a\n    b\nc\n" > no_error.target
    popd
    [ -d tab_with_tab_width ] || mkdir -p tab_with_tab_width
    pushd tab_with_tab_width
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = tab\ntab_width = 4\n" > .editorconfig
        echo -ne "a\n    b\nc\n" > no_error.target
        echo -ne "a\n  b\nc\n" > error_2.target
    popd
    [ -d invalid_value ] || mkdir -p invalid_value
    pushd invalid_value
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = invalid\n" > .editorconfig
        echo -ne "a\n b\nc\n" > invalid.target
    popd
    [ -d zero ] || mkdir -p zero
    pushd zero
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = 0\n" > .editorconfig
        echo -ne "a\n b\nc\n" > zero.target
    popd
popd

# tab_width
[ -d tab_width ] || mkdir -p tab_width
pushd tab_width
    [ -d minimum ] || mkdir -p minimum
    pushd minimum
        echo -ne "root = true\n[*.target]\ntab_width = 1\n" > .editorconfig
        echo -ne "root\n" > target.target
    popd
    [ -d numeric_indent_size ] || mkdir -p numeric_indent_size
    pushd numeric_indent_size
        echo -ne "root = true\n[*.target]\nindent_style = space\nindent_size = 4\ntab_width = 2\n" > .editorconfig
        echo -ne "root\n    child\n" > no_error.target
    popd
    [ -d indent_size_tab ] || mkdir -p indent_size_tab
    pushd indent_size_tab
        echo -ne "root = true\n[*.target]\nindent_style = tab\nindent_size = tab\ntab_width = 4\n" > .editorconfig
        echo -ne "root\n\tchild\n\t\tgrandchild\n" > no_error.target
    popd
    [ -d unset/nested ] || mkdir -p unset/nested
    pushd unset
        echo -ne "root = true\n[*.target]\nindent_style = tab\nindent_size = tab\ntab_width = 8\n" > .editorconfig
        pushd nested
            echo -ne "[*.target]\ntab_width = UnSeT\n" > .editorconfig
            echo -ne "root\n\tchild\n" > no_error.target
        popd
    popd
    [ -d zero ] || mkdir -p zero
    pushd zero
        echo -ne "root = true\n[*.target]\ntab_width = 0\n" > .editorconfig
        echo -ne "root\n" > target.target
    popd
    [ -d negative ] || mkdir -p negative
    pushd negative
        echo -ne "root = true\n[*.target]\ntab_width = -1\n" > .editorconfig
        echo -ne "root\n" > target.target
    popd
popd

# trim_trailing_whitespace
[ -d trim_trailing_whitespace ] || mkdir -p trim_trailing_whitespace
pushd trim_trailing_whitespace
    echo -ne "root = true\n[*.target]\ntrim_trailing_whitespace = true\n" > .editorconfig
    echo -ne "a\nb\nc\n" > no_error.target
    echo -ne "a\nb  \nc\n" > error.target
popd

# max_line_length
[ -d max_line_length ] || mkdir -p max_line_length
pushd max_line_length
    [ -d 10 ] || mkdir -p 10
    pushd 10
        echo -ne "root = true\n[*.target]\nmax_line_length = 10\n" > .editorconfig
        echo -ne "a\nbbbbbbbbbb\nc\n" > no_error.target
        echo -ne "a\nbbbbbbbbbbbb\nc\n" > error.target
    popd
popd

# insert_final_newline
[ -d insert_final_newline ] || mkdir -p insert_final_newline
pushd insert_final_newline
    [ -d true ] || mkdir -p true
    pushd true
        echo -ne "root = true\n[*.target]\ninsert_final_newline = true\n" > .editorconfig
        echo -ne "a\nb\nc\n" > no_error.target
        echo -ne "a\nb\nc" > error.target
        echo -ne "a\n" > single_line_no_error.target
        echo -ne "a" > single_line_error.target
    popd
    [ -d false ] || mkdir -p false
    pushd false
        echo -ne "root = true\n[*.target]\ninsert_final_newline = false\n" > .editorconfig
        echo -ne "a\nb\nc" > no_error.target
        echo -ne "a\nb\nc\n" > final_newline_no_error.target
    popd
    [ -d empty ] || mkdir -p empty
    pushd empty
        echo -ne "root = true\n[*.target]\ninsert_final_newline = true\n" > .editorconfig
        : > no_error.target
    popd
    [ -d unset ] || mkdir -p unset
    pushd unset
        echo -ne "root = true\n[*.target]\ninsert_final_newline = true\n" > .editorconfig
        mkdir -p child
        echo -ne "[*.target]\ninsert_final_newline = unset\n" > child/.editorconfig
        echo -ne "a" > child/no_error.target
    popd
    [ -d uppercase_true ] || mkdir -p uppercase_true
    pushd uppercase_true
        echo -ne "root = true\n[*.target]\ninsert_final_newline = TRUE\n" > .editorconfig
        echo -ne "a" > error.target
    popd
    [ -d end_of_line ] || mkdir -p end_of_line
    pushd end_of_line
        [ -d lf ] || mkdir -p lf
        pushd lf
            echo -ne "root = true\n[*.target]\nend_of_line = lf\ninsert_final_newline = true\n" > .editorconfig
            echo -ne "a\nb\n" > no_error.target
        popd
        [ -d crlf ] || mkdir -p crlf
        pushd crlf
            echo -ne "root = true\n[*.target]\nend_of_line = crlf\ninsert_final_newline = true\n" > .editorconfig
            echo -ne "a\r\nb\r\n" > no_error.target
        popd
        [ -d cr ] || mkdir -p cr
        pushd cr
            echo -ne "root = true\n[*.target]\nend_of_line = cr\ninsert_final_newline = true\n" > .editorconfig
            echo -ne "a\rb\r" > no_error.target
        popd
    popd
popd
