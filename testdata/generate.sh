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

# charset
[ -d charset ] || mkdir -p charset
pushd charset
    [ -d utf8 ] || mkdir -p utf8
    pushd utf8
        echo -ne "root = true\n[*.target]\ncharset = utf-8\n" > .editorconfig
        printf '\x63\x61\x66\xc3\xa9\x0a' > no_error_non_ascii.target
        printf '\xef\xbb\xbf\x63\x61\x66\xc3\xa9\x0a' > error_bom.target
        printf '\xc3\x28\x0a' > error_invalid_sequence.target
    popd
    [ -d utf8_bom ] || mkdir -p utf8_bom
    pushd utf8_bom
        echo -ne "root = true\n[*.target]\ncharset = UTF-8-BOM\nend_of_line = lf\n" > .editorconfig
        printf '\xef\xbb\xbf\x63\x61\x66\xc3\xa9\x0a' > no_error_non_ascii.target
        printf '\x63\x61\x66\xc3\xa9\x0a' > error_missing_bom.target
        : > error_empty.target
    popd
    [ -d latin1 ] || mkdir -p latin1
    pushd latin1
        echo -ne "root = true\n[*.target]\ncharset = latin1\n" > .editorconfig
        printf '\x63\x61\x66\xe9\x0a' > no_error_non_ascii.target
    popd
    [ -d utf16be ] || mkdir -p utf16be
    pushd utf16be
        echo -ne "root = true\n[*.target]\ncharset = utf-16be\n" > .editorconfig
        printf '\x00\x63\x00\x61\x00\x66\x00\xe9\x00\x0a' > no_error_non_ascii.target
        printf '\x00' > error_odd_byte_length.target
    popd
    [ -d utf16le ] || mkdir -p utf16le
    pushd utf16le
        echo -ne "root = true\n[*.target]\ncharset = utf-16le\n" > .editorconfig
        printf '\x63\x00\x61\x00\x66\x00\xe9\x00\x0a\x00' > no_error_non_ascii.target
        printf '\x00' > error_odd_byte_length.target
    popd
    [ -d unset ] || mkdir -p unset/nested
    pushd unset
        echo -ne "root = true\n[*.target]\ncharset = utf-8\n" > .editorconfig
        echo -ne "[*.target]\ncharset = unset\n" > nested/.editorconfig
        printf '\x63\x61\x66\xe9\x0a' > nested/no_error_latin1.target
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
    popd
    [ -d false ] || mkdir -p false
    pushd false
        echo -ne "root = true\n[*.target]\ninsert_final_newline = false\n" > .editorconfig
        echo -ne "a\nb\nc" > no_error.target
    popd
popd
