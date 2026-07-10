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
        echo -ne "a\n\t\tb\nc\n" > error_tab.target
    popd
    [ -d tab ] || mkdir -p tab
    pushd tab
        echo -ne "root = true\n[*.target]\nindent_style = tab\n" > .editorconfig
        echo -ne "a\n\t\tb\nc\n" > no_error.target
        echo -ne "a\n  b\nc\n" > error_space.target
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
    [ -d 1 ] || mkdir -p 1
    pushd 1
        echo -ne "root = true\n[*.target]\nmax_line_length = 1\n" > .editorconfig
        echo -ne "a\n\n" > no_error.target
        echo -ne "ab\n" > error.target
    popd
    [ -d 10 ] || mkdir -p 10
    pushd 10
        echo -ne "root = true\n[*.target]\nmax_line_length = 10\n" > .editorconfig
        echo -ne "a\nbbbbbbbbbb\nc\n" > no_error.target
        echo -ne "a\nbbbbbbbbbbbb\nc\n" > error.target
    popd
    [ -d tabs ] || mkdir -p tabs
    pushd tabs
        echo -ne "root = true\n[*.target]\nmax_line_length = 4\n" > .editorconfig
        echo -ne "\\t\\t\\t\\t\n" > no_error.target
        echo -ne "\\t\\t\\t\\t\\t\n" > error.target
    popd
    [ -d multibyte ] || mkdir -p multibyte
    pushd multibyte
        echo -ne "root = true\n[*.target]\nmax_line_length = 5\n" > .editorconfig
        echo -ne "あいうえお\n" > no_error.target
        echo -ne "あいうえおか\n" > error.target
    popd
    [ -d crlf ] || mkdir -p crlf
    pushd crlf
        echo -ne "root = true\n[*.target]\nend_of_line = crlf\nmax_line_length = 4\n" > .editorconfig
        echo -ne "aaaa\\r\\n" > no_error.target
        echo -ne "aaaaa\\r\\n" > error.target
    popd
    [ -d unset ] || mkdir -p unset
    pushd unset
        echo -ne "root = true\n[*.target]\nmax_line_length = 1\n" > .editorconfig
        [ -d child ] || mkdir -p child
        pushd child
            echo -ne "[*.target]\nmax_line_length = UNSET\n" > .editorconfig
            echo -ne "this line is deliberately longer than one character\n" > no_error.target
        popd
    popd
    [ -d invalid ] || mkdir -p invalid
    pushd invalid
        for value in 0 -1 1.5 invalid; do
            [ -d "$value" ] || mkdir -p -- "$value"
            echo -ne "root = true\n[*.target]\nmax_line_length = $value\n" > "$value/.editorconfig"
            echo -ne "a\n" > "$value/target.target"
        done
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
    popd
    [ -d false ] || mkdir -p false
    pushd false
        echo -ne "root = true\n[*.target]\ninsert_final_newline = false\n" > .editorconfig
        echo -ne "a\nb\nc" > no_error.target
    popd
popd
