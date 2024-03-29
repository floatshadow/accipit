# 浙江大学24年春夏编译原理实验

仓库目录结构：

```bash
├── examples/   # 样例
├── docs/       # 实验文档   
├── mkdocs.yml
└── src/        # 解释器源码
└── tests/      # 各 lab 的测试输入
```

参考材料:

- [样例编译器](https://git.zju.edu.cn/accsys/accsys-rs) (尚未完工)
- [PEG 和 Parser Combinator 的样例](https://git.zju.edu.cn/accsys/peg-test)
- [SysY 运行时库](https://git.zju.edu.cn/accsys/sysy-runtime-lib)
- [CMake 项目模板](https://git.zju.edu.cn/accsys/accsys-cmake-template)

## 实验介绍
具体来说整个实验分为五个小实验：

- 环境配置与测试用例编写：配置实验环境，学习 SysY 语法。
- 词法分析与语法分析： 实现词法分析和语法分析，将源代码转化为语法树。
- 语义分析：实现符号表，基于语法树进行语义分析。
- 中间代码生成：把分析后的语法树转化为实验定义的中间代码。
- 目标代码生成：将中间代码转化为 RISC-V 64 汇编代码。

## 致谢

我们对本课程设计中参考的课程与资料表示感谢：

- [全国大学生计算机系统能力大赛](https://compiler.educg.net/#/) 要求实现的 SysY 语言和大量测试用例来自于该大赛。
- [南京大学编译原理](https://cs.nju.edu.cn/changxu/2_compiler/index.html) 我们的部分文档参考了该课程的文档。我们也参考了该课程设计的某些测试用例。
- [北京大学编译原理](https://pku-minic.github.io/online-doc/#/) 我们的部分文档参考了该课程的文档。
- [浙江大学编译原理](https://compiler.pages.zjusct.io/sp24/) 我们的部分文档参考了隔壁班 (刘忠鑫老师) 的文档。