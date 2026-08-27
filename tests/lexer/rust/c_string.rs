#!emit tokens
c"c string"

c"all escapes in one line : \" \' \x32 \n \r \t \\ \0"

c"
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

c"
    multi
    line
    string
"

c"multi\
line\
c\
string\
with\
these\
"
    
c"c string"suffix

c"all escapes in one line : \" \' \x32 \n \r \t \\ \0 "suffix

c"
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

c"
    multi
    line
    c
    string
"suffix

c"multi\
line\
c\
string\
with\
these\
"suffix
