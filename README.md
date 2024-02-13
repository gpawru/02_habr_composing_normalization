# NF(K)C нормализация строк Unicode

примеры для статьи на Хабре: _вставить ссылку_

### структура репозитория:

- [**benches**](benches) - бенчмарки нормализации
- [**tests**](tests) - тесты нормализации
- **data** - "запечённые" данные декомпозиции
- [**composing**](composing) - нормализация строк
- [**source**](source) - данные UCD и их парсинг
- [**prepare**](prepare) - экспорт данных в файлы
- **test_data** - данные для тестирования и бенчмарков

запуск бенчмарков:

```
make bench
```
*(результат - в виде CSV)*

