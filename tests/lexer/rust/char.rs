#! --test=emit-tokens
// char literal unsuffixed
'a'
// quote escape unsuffixed
'\''
'\"'
// ascii escape unsuffixed
'\x32'
'\n'
'\r'
'\t'
'\\'
'\0'
// unicode escape unsuffixed
'\u{123123}'
'\u{46abef}'
'\u{deadbe}'

// char literal suffixed
'a'char
// quote escape suffixed
'\''char
'\"'char
// ascii escape suffixed
'\x32'char
'\n'char
'\r'char
'\t'char
'\\'char
'\0'char
// unicode escape suffixed
'\u{123123}'char
'\u{46abef}'char
'\u{deadbe}'char

