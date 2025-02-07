# Тестовое задание для Bitsgap Holding

## Структура проекта
- `shared` - общий код, который понадобится для работы с разными биржами
- `poloniex` - код специфичный для биржи
- `scraper` - бинарник который скачивает исторические и реалтайм данные в монгу
- `musings.md` - журнал разработки в стиле потока сознания

## Подготовка
- Запускаем монгу и настраиваем новую БД с пользователем
    - Я у себя сделал так:
    ```bash
    systemctl enable mongodb
    sudo systemctl start mongodb
    mongo
    use bitsgap_qthree_test
    db.createUser({ user: "scraper", pwd: "scraper", roles: [ { role: "readWrite", db: "bitsgap_qthree_test" } ] })
    ```
    - `scraper` при каждом запуске базу дропает сам
- Получаем ключи для API poloniex
    - Устанавливаем переменные среды
    ```bash
    export API_KEY="$API_KEY"
    export SECRET_KEY="$SECRET_KEY"
    ```
## Запуск тестов
- Вызываем в корне проекта
```
cargo test -- --nocapture
```

## Запуск scraper
- Вызываем в корне проекта
```
cargo run --release --bin bitsgap_scraper -- --mongodb-uri mongodb://scraper:scraper@localhost/bitsgap_qthree_test
```
