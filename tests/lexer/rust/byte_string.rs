#!emit tokens
b"byte string"

b"all escapes in one line : \" \' \x32 \n \r \t \\ \0"

b"
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

b"
    multi
    line
    string
"

b"multi\
line\
string\
with\
these\
"
    
b"byte string"suffix

b"all escapes in one line : \" \' \x32 \n \r \t \\ \0 "suffix

b"
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

b"
    multi
    line
    string
"suffix

b"multi\
line\
string\
with\
these\
"suffix
