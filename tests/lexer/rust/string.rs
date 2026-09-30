#! --test=emit-tokens
"string"

"all escapes in one line : \" \' \x32 \n \r \t \\ \0"

"
all escapes in multiple lines :
\"
\'
\x32
\n
\r
\t
\\
\0
"

"
    multi
    line
    string
"

"multi\
line\
string\
with\
these\
"
    
"string"suffix

"all escapes in one line : \" \' \x32 \n \r \t \\ \0 "suffix

"
all escapes in multiple lines :
\"
\'
\x32
\n
\r
\t
\\
\0
"suffix

"
    multi
    line
    string
"suffix

"multi\
line\
string\
with\
these\
"suffix
