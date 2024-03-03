%{
#include <stdio.h>
#include <ctype.h>
#include <math.h>

#define YYPARSE_PARAM scanner
#define YYLEX_PARAM   scanner

int _is_eof; 
int return_state;
%}

%define api.pure

%token INT FLOAT
%start print
%union {
    double _fp_type;
    int _int_type;
}
%type <_int_type> INT
%type <_fp_type> FLOAT exp term factor power

%left '+' '-'
%left '*' '/'
%left UMINUS UPLUS
%right '^'

%% 
print : exp { printf("ans = %lf\n", $1); }
      |
      ;

exp : exp '+' term      { $$ = $1 + $3; }
    | exp '-' term      { $$ = $1 - $3; }
    | '-' exp %prec UMINUS { $$ = -$2; }
    | '+' exp %prec UPLUS  { $$ =  $2; }
    | term              { $$ = $1; }
    ;

term : term '*' power  { $$ = $1 * $3; }
    | term '/' power   { $$ = $1 / $3; } 
    | power            { $$ = $1; }
    ;

power : factor '^' power { $$ = pow($1, $3); }
      | factor           { $$ = $1; }
      ;

factor : '(' exp ')'    { $$ = $2;}
       | INT            { $$ = $1;}
       | FLOAT          { $$ = $1;}
       ;
%%


int yyerror(char *s) {
    fprintf(stderr, "%s\n", s);
    return 0;
}

int main() {
    return_state = 0;
    _is_eof = 0;
    while (!_is_eof) {
        return_state = yyparse();
    }
    return 0;
}
