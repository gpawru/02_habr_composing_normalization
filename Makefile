# бенчмарки
bench:
	cd benches && cargo bench >> report.txt && cargo run report.txt && rm report.txt

# тесты
test:
	cd tests && cargo test
