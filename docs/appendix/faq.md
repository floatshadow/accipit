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

## 为什么 Accipit IR 有 `alloca` 指令但是没有 `free`

Accipit IR 的定位是平台无关的中间代码，在**显式**地表达前端语义的同时，在一些形式又接近底层的汇编（例如控制流跳转，指令的操作码等）.

`alloca` 指令的语义是分配栈上的空间，用于存放局部变量.
`alloca` 指令告诉了编译器后端局部变量需要的空间，并在汇编中由函数的 prologue 部分完成，即 `sub sp, sp, #size`.
在函数体中，使用 `sp + #offset` 的形式访问局部变量的地址.
因此在退出函数时，epilogue 部分复原栈指针即完成了释放局部变量空间的动作.