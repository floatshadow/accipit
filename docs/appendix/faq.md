# FAQ

## 为什么我的 lexer/parser 卡住了

flex/bison 默认从 `stdin` 读入输入, 由于 `\n` 也会视为输入的一部分, 你需要手动输入 `EOF` 结束, 这在 `bash` 上是 `Ctrl+D`. 

如果你想从文件里输入的话 (比如 `./compiler test.in`), 这里提供一个供参考的解决方法: 

```cpp
extern int yyparse();

extern FILE* yyin;

int main(int argc, char **argv) {
	yyin = fopen(argv[1], "r");
    yyparse();
    return 0;
}
```