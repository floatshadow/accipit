<style>
h2:before {
	content: none;
}
</style>

# 更新日志

## 2024-4-28

- 修复了 lab1 和 lab2 测试中一些函数没有返回值的问题 (虽然这符合语法和语义, 但这是 Undefined Behavior, 我们会在测试中避免) [commit#4e94a38](https://git.zju.edu.cn/accsys/accipit/-/commit/4e94a38f0adc7a96eafd09cdb8f0574363fcec32)

## 2024-4-27

- 修正了 lab1 测试中 `func_array1.sy` 符合语法但不符合语义的问题 [commit#4c00772](https://git.zju.edu.cn/accsys/accipit/-/commit/4c00772bb3334f5e918214bec2459cd16a374d3c)
