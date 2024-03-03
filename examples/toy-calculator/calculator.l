%{
#include <stdio.h>
#include "calculator.tab.h"   


static int nested_paras = 0;
extern int _is_eof;
%}

%option noyywrap
%option bison-bridge

DIGIT   [0-9]
OP      [\+\-\*\/\^]
%s PARAS

%%
{DIGIT}+ {
    sscanf(yytext, "%d", &yylval->_int_type);
    return (INT);
}

{DIGIT}*"."{DIGIT}+ {
    sscanf(yytext, "%lf", &yylval->_fp_type);
    return (FLOAT); 
}

"(" {
    BEGIN(PARAS);
    nested_paras++;
    return '(';
}

<PARAS>")" {
    if (--nested_paras == 0) BEGIN(INITIAL);
    return ')';
}

{OP} {
    return yytext[0];  
}

<<EOF>> {
    _is_eof = 1;
    yyterminate();
}

\n {
    yyterminate();
}


. {
    fprintf(stderr, "unrecogized input %s\n", yytext);
}
%%