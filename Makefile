# бенчмарки
bench:
	cd benches && cargo bench opt && cargo bench icu && cargo bench base

# тесты
test:
	cd tests && cargo test

# очистка
clean:
	cd benches && cargo clean
	cd prepare && cargo clean
	cd source && cargo clean
	cd tests && cargo clean
	cd decomposing/1_base && cargo clean
	cd decomposing/2_opt && cargo clean

# подготовка данных
bake:
	cd prepare && cargo run
