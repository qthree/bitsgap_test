# 2025-02-04

- в ТЗ: `Начинаем получать RT (на бирже Trades) по WS и собирать новые KL`
    - Получать KL по WS или вычислять их самому на основе RT?

- в ТЗ: `Список таймфреймов – 1м, 15м, 1ч, 1д`
    - Хранить в базе KL для каждого интервала?
    - Или хранить только для 1м, а для других вычислять на лету в запросе к БД? Если БД такое позволяет.
    - Или хранить вообще только RT и на основе их вычислять какждый раз?
    - Лучшее решение зависит от того как часто будут запросы к 1ч-1д KL, если часто то лучше хранить их
        - Чтобы каждый раз заново не рассчитывать
        - Всё равно место в БД 1ч-1д будут занимать сильно меньше чем 1м KL
        - Сохраняем 1ч-1д KL, можно оптимизировать потом.
    - Так же завист от количества RT в минуту
        - Если часто будут минуты когда не было ни одной сделки - будет сохраняться много "пустых" KL
            - Надо ли сохранять эти пустые KL?
            - Сохраняем пустые KL, можно оптимизировать потом.
        - Если будет слишком много запросов в минуту, то даже 1м KL будет дорого рассчитывать
            - Сохраняем 1м-15м, можно будет оптимизировать потом.
    - В тестовом проекте не идёт речь о запросах данных из БД, только о наполнении её данными.
        - В любом случае, пока заполняем БД всеми пришедшими данными
        - т.к. единственный способ проверить корректность данных пока - с помощью сторонних инструментов БД
            - и тестов?
            - Было бы неплохо написать тесты для проверки содержимого БД, хоть чтение из БД в проект не входит
            - Получается придётся в любом случае добавить минимальную реализацию чтения из БД?

- В интервью просили сделать решение "масштабируемым"
    - Если речь о производительности
        - Rust и Tokio уже предоставляют удовлитворительную производительность, а точнее минимальный overhead
        - Предусмотреть использование БД которую можно масштабировать
            - Количество операций записи не будет зависеть от количества пользователей агрегатора
                - Не требуется возможность писать в разные инстансы БД
                - Не требуется алгоритм консенсуса
                - Достаточно чтобы данные текли от основного инстанса в зеркала
            - Нужно изучить характер данных
                - Возможно в реальном проекте вместо масштабирования БД можно будет обойтись (частично?) кэшированием данных in-memory приложения
            - В интервью спрашивали про знания о Tarantool
                - Есть планы использовать его в реальном проекте?
                - Можно попробовать использовать его
            - Предусмотреть возможность использовать разных БД в качестве бэкенда для хранения данных?
                - Можно отрефакторить потом при надобности
                - Слишком out-of-scope
                - Трудно заранее сформулировать подходящий уровень абстракции для абсолютно разных БД (MongoDB vs PostreSQL, например)
        - В тестовом проекте не идёт речь о запросах данных из БД, только о наполнении её данными
            - Производтельность получение данных от API бирж никак не зависит от количества пользователей агрегатора, и в целом не является проблемой.
            - Заботиться о производительности придётся только при имплементации REST API агрегатора и чтения из БД
    - Если речь о использовании API разных бирж
        - Предусмотреть струтктуру проекта которая позволяет подключать новые биржи
            - с наименьшим boilerplate
            - при этом инкапсулировать переиспользуемые код и код для разных бирж

- Структура проекта
    - Библиотека с общим кодом
        - В общей библиотеке должны быть универсальные типы которые сохраняются в БД, с сигнатурой как в ТЗ
        - Можно принить к этим универсальным типам derive serde::Serialize, если нужна сериализация в JSON
            - Eсли будем отдавать их по своему REST API в полноценном проекте
            - Если будем сохранять как json в БД (MongoDB?)
    - По отдельной библиотеке на каждую биржу, зависят от общей библиотеки
        - В отдельных библиотеках живут типы специфичные для соответствующего REST API
        - Применяем к этим специфичным типам derive serde::Deserialize чтобы парсить ответ от REST API
        - Можно конвертировать специфичные типы в универсальный с помощью моего крейта derive_convert
    - Исполняемый крейт для получения и сохранения данных
        - Включает в себя крейты всех бирж, который можно включать/выключать при компиляции (features) и в рантайме (конфиг)
    - В будущем возможны другие исполняемые крейты которые будут выполнять функции (чтение из БД, торговые операции)
        - которые смогут переиспользовать те же библиотеки с общим кодом и с кодом специфичным для бирж

- Типы данных
    - В реальном проекте стоит обсудить использование внутренних типов вместо String
    - Строгая типизация поможет избежать логических ошибок
        - Сделает невозможным хранить в памяти невалидный стейт
        - После первичного парсинга мы будем уверены что данная запись всегда валидна
    - Нужно понять какие типы должны иметь возможность меняться/дополняться без перекомпиляции
    - `struct RecentTrade`
        - `tid: String` -> тип гарантирующий валидный формат Id
        - `pair: String` -> `(String, String)`, опционально с обёрткой, которая гарантирует что мы поддерживаем такую валютную пару
        - `price: String` и `amount: String` -> число не с плавающей точкой
            - см. crate [`fastnum`](https://docs.rs/fastnum/latest/fastnum/)
            - или [`Ratio<i128>`](https://docs.rs/num/latest/num/rational/struct.Ratio.html) в `num`
        - `side: String` -> `enum Side { Buy, Sell }`
        - `timestamp: i64` -> обёртка с методами для конвертации в типы данных из std/time/chrono
    - `struct Kline`
        - `time_frame: String`
            - Фиксированные значения -> `enum TimeFrame { Minute, Minute15, Hour, Day }`
            - Динамическое значение (мы не знаем какие могут быть TF на будущих биржах) -> аналог `Duration`
        - `o, h, l, c, volume_bs`: `f64` -> число не с плавающей точкой?
            - или лучше прокинуть насквозь как есть, чтобы избежать потерь при конверсии туда-обратно?
    - При сохранении в БД можно сохранять по прежнему как String
        - т.е. эти изменения никак не коснутся результата в БД, только организацию внутри программы
    - в ТЗ указаны конкретные сигнатуры типов, оставим пока как есть, без парсинга String, потом можно отрефакторить.

- Обработка ошибок
    - В библиотеках желательно использовать thiserror
    - Но для тестового прототипа пойдёт и anyhow
    - Позже можно отрефакторить, когда будут известны основные типы ошибок.

- Инструменты в shared крейте
    - `ApiRequester`
        - Отправка GET запросов и парсинг json ответа
        - Аутентификация с `enum AuthMethod`
            - Позволит испольховать на разных биржах схожие способы аутентификации без дублирования кода
            - На poloniex.com используется HMAC-SHA256
            - В хэдеры каждого запроса помещается sign который генерируется из параметров запроса
                - От timestamp запроса он не зависит
                - Можно было бы сгенерировать его всего один раз для запроса с неизменным url и параметрами
                - Но оно того вряд ли стоит
            - Наивная имплементация подразумевает что если на других биржах тоже есть HMAC-SHA256 auth, то там используются точно такие же хэдеры
                - Разумеется это не так, но пока сложно выбрать подходящую абстракцию, чтобы поддержать HMAC-SHA256 с разными названиями и характеристиками хэдеров
                - Возможно придётся делать отдельный набор `AuthMethod` для каждой биржи, и делать инкапсуляцию на уровне функций, которые уже вызывать из методов `AuthMethod`
                - Потребуется рефакторинг.

- Настройка
    - Возможно, если делать внешний конфиг, например в формате toml, нужно разделить "биржу" на две сущности
        - "API"
            - credentials
            - выбор какой модуль отвечает за парсинг ответов, конвертацию их в универсальные
            - не знает ничего о сборе RT и KL
        - "Scraper"
            - непосредственно настройка сборщика RT и KL
            - указывает на один из "API"
        - Возможны произвольные комбинации API и Scraper
            - Несколько Scraper используют один и тот же API, но с разными настройками, например разные пары, или разные промежутки KL.
            - Несколько API для одной и той же биржи, по одному Scraper (или по одному торговому боту) на каждый, чтобы уменьшить количество обращений по одному ключу.
                - В реальном проекте же у каждого клиента будут свои credentials
            - Возможно, один scraper и множество API у него? Чтобы переиспользовать настройки одного scraper через множество API?
    - Предусмотреть, что в реальном проекте, при настройке торговых ботов, потребуется реалтайм добавление/удаление конфигов (API, боты)

- За первый день успел сделать
    - Отправка GET запроса по произвольному пути к API poloniex
    - Формирование auth payload, подписание приватным ключом, добавление в хэдеры запроса
    - Получение api и secret key из ENV VAR
    - Тест в shared крейте который делает запрос к ручке "/markets" и печатает pretty printed json value
    - Сработало с первого запуска
```json
[
  {
    "baseCurrencyName": "BTS",
    "crossMargin": {
      "maxLeverage": 1,
      "supportCrossMargin": false
    },
    "displayName": "BTS/BTC",
    "quoteCurrencyName": "BTC",
    "state": "NORMAL",
    "symbol": "BTS_BTC",
    "symbolTradeLimit": {
      "amountScale": 8,
      "highestBid": "0",
      "lowestAsk": "0",
      "minAmount": "0.00001",
      "minQuantity": "100",
      "priceScale": 10,
      "quantityScale": 0,
      "symbol": "BTS_BTC"
    },
    "tradableStartTime": 1659018816626,
    "visibleStartTime": 1659018816626
  },
  ...
]
```

# 2025-02-05

## TODO
- [x] Запрос KL ~~и RT~~ по REST API
- [ ] Написать или [сгенерировать](https://transform.tools/json-to-rust-serde) специфичные для poloniex типы, для парсинга ответа REST ручек.
    - [x] KL
    - ~~[ ]RT~~
- [ ] Скопировать из ТЗ универсальные типы, добавить конверсию из специфичных типов
    - [x] KL
    - [ ] RT
- [ ] Заимплементить получение данных по WS
    - в ТЗ говорится что мы сначало получаем данные по REST, а потом по WS
    - возможно стоит делать в обратном порядке?
    - нужно убедиться что между REST запросом и подпиской на WS мы не пропустим события RT
    - если мы сначало сделаем подписку на WS, то можем получить дублирующиеся RT, но это решить проще
    - Нужно проверить на реальных данных.

## Recent trades
- У ручки нет указания временного промежутка, только последние N событий
- Что в принципе логично, не зря это RECENT trades
- `Интервал времени – 2024-12-01 – текущее время`
    - Не правильно понял ТЗ?
    - Видимо это только для KL
- `Начинаем получать RT (на бирже Trades) по WS`
    - Ну да, по REST значит получаем только KL, а RT только по WS
    - Имплементацию WS отложу на потом

## Candles (KL)
- Нужно собирать для `BTC_USDT, TRX_USDT, ETH_USDT, DOGE_USDT, BCH_USDT`
    - названия из ТЗ совпдают с "symbol" в выводе "/markets"
    - значит названия из ТЗ не нужно преобразовывать, как есть скармливать запросу
- Жалко что нельзя запрашивать сразу для нескольких symdol в одном запросе
- Насколько позволительно запросить все symbolы параллельно?
    - Последовательно будет медленно
    - Параллельно может упереться в rate limit
        - У `/markets/{symbol}/candles` ограниченте на 200 запросов в секунду
    - Встроить в ApiRequester ограничитель количества одновременных запросов?
        - Или количтва запросов в секунду?
        - Но ApiRequester это низкоуровневая абстракция которая не знает у какой ручки какой rate limit
    - Мы делаем запросы к candles только при старте программы, и всего 5, по запросу на symbol
    - Идеальное решение сейчас трудно сформулировать, для тестового проекта не является проблемой
    - Отложить на потом, пока можно делать запросы параллельно без rate limit.
- Интервалы
    - Снова встаёт вопрос: нужна ли возможность обновлять API биржи без пересборки кода
        - Например, добавлять/менять доступные для данной биржи интервалы

# WIP
- Вместо того чтобы делать ТЗ увлёкся вспомогательным кодом
    - trait BuildUrl для эффективного и удобного создания запросов
        - можно было бы использовать частично serde крейты для работы с url
            - где то было бы более удобно, но не так гибко
        - но например если нужен некий контекст для сериализации то serde не подойдёт
            - словарь для конвертации параметров запроса между разными биржами, например
        - нужно будет подумать когда наберётся больше примеров использования
        - заменять ли на что-то, или дальше развивать
            - например, свой derive написать
            - или использовать тот что уже [есть](https://github.com/qthree/represent)
        - Пока оставлю.
    - SortedVec чтобы хранить динамический словарь названий интервалов
    - Надо переходить ближе к делу.

- ручка `candles` работает (пока без парсинга ответа)
```json
[
  [
    "98113.42",
    "98177.78",
    "98171.66",
    "98113.42",
    "77317.88",
    "0.787699",
    "34974.58",
    "0.356324",
    88,
    1738770236995,
    "98156.63",
    "MINUTE_1",
    1738770180000,
    1738770239999
  ],
  ...
]
```
- парсинг ответа `candles` в специфичную для биржи структуру работает
```json
[
    CandlesResponse {
        low: "0.22485",
        high: "0.22485",
        open: "0.22485",
        close: "0.22485",
        amount: "3635.01571",
        quantity: "16166.403",
        buy_taker_amount: "1849.58147",
        buy_taker_quantity: "8225.846",
        trade_count: 296,
        timestamp: 1738770240221,
        weighted_average: "0.22485",
        interval: "MINUTE_1",
        start_time: 1738770180000,
        close_time: 1738770239999,
    },
    ...
]
```
- Делаю конвертацию из специфичного KL ответа с биржи в тип для БД из ТЗ
    - Желательно сделать типы которые автоматом парсят строки в нужные типы и обратно
        - т.е. хранят in-memory распаршенные значения
        - и прозрачно конвертируют при записи в БД
        - Но пока нет на это времени.
    - `impl TryFrom<CandlesResponse>` для типа как в ТЗ сделать не получится
        - нужен контекст из запроса
            - название пары, его нет в ответе
        - нужен глобальный контекст
            - если нужно конфигурируемое название пар в БД
        - пока сделаю как простую функцию CandlesResponse
            - потом продумаю сигнатуру трейта для конверсии между типами биржи и БД
    - Возник вопрос откуда брать VBS
        - т.е. если я правильно понимаю, надо вот это
        ```rust
        /// quote units traded over the interval
        pub amount: String,
        /// base units traded over the interval
        pub quantity: String,
        /// quote units traded over the interval filled by market buy orders
        pub buy_taker_amount: String,
        /// base units traded over the interval filled by market buy orders
        pub buy_taker_quantity: String,
        ```
        - превратить в это
        ```rust
        /// объём покупок в базовой валюте
        pub buy_base: f64,
        /// объём продаж в базовой валюте
        pub sell_base: f64,
        /// объём покупок в котируемой валюте
        pub buy_quote: f64,
        /// объём продаж в котируемой валюте
        pub sell_quote: f64,
        ```
        - Выглядит так, что amount и quantity - это Total
        - а buy_taker_amount и buy_taker_quantity это часть от Total
        - тогда получаем что-то вроде:
        ```rust
            buy_base: buyTakerQuantity,
            sell_base: quantity - buyTakerQuantity,
            buy_quote: buyTakerAmount,
            sell_quote: amount - buyTakerAmount,
        ```
        - Нужно убедиться.
- конвертация ответа `candles` в тип как в ТЗ работает (осталось уточнить по VBS)
```rust
{"pair":"BTC_USDT", "time_frame":"1m","o":98019.95,"h":98062.32,"l":97990.6, "c":98057.02,"utc_begin":1738770720000,"volume_bs":{"buy_base":0.218558, "sell_base":0.40820199999999995,"buy_quote":21427.53,   "sell_quote":40020.700000000004}}
{"pair":"TRX_USDT", "time_frame":"1m","o":0.22484, "h":0.22485, "l":0.22484, "c":0.22485, "utc_begin":1738770720000,"volume_bs":{"buy_base":13875.81, "sell_base":12273.219,          "buy_quote":3119.93633, "sell_quote":2759.5826199999997}}
{"pair":"ETH_USDT", "time_frame":"1m","o":2764.21, "h":2768.29, "l":2763.69, "c":2766.97, "utc_begin":1738770720000,"volume_bs":{"buy_base":5.124448, "sell_base":3.7449640000000004, "buy_quote":14176.29,   "sell_quote":10361.969999999998}}
{"pair":"DOGE_USDT","time_frame":"1m","o":0.262531,"h":0.262976,"l":0.262451,"c":0.262668,"utc_begin":1738770720000,"volume_bs":{"buy_base":54178.956,"sell_base":42764.02300000001,  "buy_quote":14232.02106,"sell_quote":11231.982370000002}}
{"pair":"BCH_USDT", "time_frame":"1m","o":333.14,  "h":333.7,   "l":333.07,  "c":333.49,  "utc_begin":1738770720000,"volume_bs":{"buy_base":7.126704, "sell_base":7.447879,           "buy_quote":2375.79,    "sell_quote":2482.96}}
```
- можно начинать делать WS или сохранение в БД.

# 2025-02-06
## TODO
- [x] Заимплементить получение данных по WS
- [x] Сделать специфичные для poloniex типы для парсинга данных из WS
- [x] Скопировать из ТЗ универсальные типы, добавить конверсию из специфичных типов для RT
- [x] Сохранять в БД все данные в формате из ТЗ

# Изменение в ТЗ
- ~~`struct VBS` -> "Достаточно заполнить buy_base: quantity"~~
- "Ещё рабята написали, что правильно структура сопоставлена"
    - воротаю всё взад.
- А вот в сообщениях WS в candles канале как раз таки нет разделения на buy и sell
    - т.е. нет buyTakerAmount и buyTakeQuantity
    - и поэтому не можем посчитать какая часть total quantity/amount приходитя на buy, а какая на sell
    - говорили ранее "Достаточно заполнить buy_base: quantity"
    - тогда пока положим total в buy_base и buy_quote
    ```rust
    VBS {
        buy_base: quantity,
        sell_base: 0.0,
        buy_quote: amount,
        sell_quote: 0.0,
    }
    ```

# WS
- ping/pong
    - Очень странно что используется Text сообщения WS с json внутри
    - вместо тогда чтобы использовать встроенные в протокол WS Ping/Pong сообщения
- Subscription Request
    - `# Failure { "event": "error",` 
        - ошибка подписки не указывает что это именно ошибка подписки
        - нет никакого id чтобы понять какой из запросов провалился
        - будет трудно использовать одно соединение в котором получаем реалайтм данные и неидетифицируемые сообщения об ошибках
        - не актуально если мы делаем подписку только при открытии соединения
        - т.к. при открытии соединения можно работать с ним в последовательном режиме
            - Отправили запрос - получили ответ
            - потом только посылаем следующий.
        - могут ли ошибки приходить произвольно, уже во время активной работы?
    - Хоть какая то идентификация таки есть
        - но это просто текст
        ```
        Subscription Errors

        Subscription failed (generic)
        Already subscribed
        Bad request (generic)
        ```
        - всё равно не понятно на какой именно запрос ошибка, если делать сразу несколько запросов пачкой
    - Даже если потом разрешать подписку/отписку в середине работы приложения
        - то нужно разграничить её с активным получением данных
        - делать запросы исключительно последовательно
            - делаем запрос
            - получаем ответ Ok/Failure
            - только потом делаем следующий
    - А ещё на каждый каждый канал приходит отдельный Ok/Failure
        - ещё и придётся считать количество ответов перед тем как послать следующий запрос
        - Очень неудобно.
- Не забываем о временных ограничениях на задание
    - трудно предсказать как будет выглядеть взаимодействие по WS с другими биржами
    - преждевременные абстракции только тратят время на разработу
    - всегда можно отрефакторить и перенести повторяющийся код в shared крейт
    - Сделаю пока механизм WS конкретно для poloniex.
- WS позволяет получать live обновления валют и маркетов
    - Можно будет в будущем на лету добавлять новые валюты/маркеты, без лишних запросов к REST.
- На каждый интервал свой канал
    - со своим названием
    - сделаю пока ещё один словарь (внутренне представление интервала) -> (название канала)
    - после добавления новых бирж ещё раз подумать, нужен ли всё таки конфиг со словарями
    - или всё таки захардкодить это всё в функции бинарника.

- У poloniex в RT (и в REST, и в WS) есть два поля с обозначением времени
    - одно указывает на время сделки, другое на время публикации записи
    ```
    createTime	Long	time the trade was created
    ts	Long	time the record was pushed
    ```
    - в ТЗ есть поле
    ```
    RecentTrade::timestamp // время UTC UnixNano
    ```
    - какое время здесь подойдёт лучше?
    - судя по всему особой разницы нет: "ts" всего на 15-35 миллисекунд позде чем "createTime"
    - Возьму пока "createTime", если что потом исправлю.

- Мы не может использовать TryFrom при конвертировании некоторых типов
    - за счёт того что нужен контекст
    - подумать в свободное время о том чтобы переписать [derive_convert](https://github.com/qthree/derive_convert)
    - так чтобы вместо TryFrom/From он имплементил новый трейт
    - который позволяет передавать контекст в метод конвертации.

- `https://api.poloniex.com/markets/{symbol}/candles`
    - максимальное количество сообщений за запроса 500 штук
        - в ТЗ просят скачать все KL начиная с 2024-12-01
        - для 1м интервала это явно больше 500
        - нужно заимплементить скачивание данных отрывками
        - и склеивать отрывки там чтобы не было повторов на стыках
    - все сообщения могут не поместиться в RAM
        - хотя вряд ли конечно
        - но что если потребуется скачать всю историю что есть на биржке
        - сохранять в БД кусками, не дожидаясь пока все куски выкачаются?
    - запоминать последний ts ("time the record was pushed") в куске
        - в следующем куске выбрасывать все сообщения в начале
        - у которых ts меньше запомненного
    - возможно не успею сделать дедупликацию
        - (проверку что в местах склейки нет повторяющихся сообщений)
        - для тестового задания не критично
        - для начала завершить требуемый функционал.

- К public ws подключение проходит
    - аутентификация не нужна
    ```rust
    Response {
        status: 101,
        version: HTTP/1.1,
        headers: {
            "date": "Thu, 06 Feb 2025 11:29:58 GMT",
            "connection": "upgrade",
            "upgrade": "Websocket",
            "sec-websocket-accept": "ef6mQXbdW+Yk2i2HZETb4O8goag=",
            "cf-cache-status": "DYNAMIC",
            "strict-transport-security": "max-age=15552000; includeSubDomains",
            "x-content-type-options": "nosniff",
            "server": "cloudflare",
            "cf-ray": "90dace2d9b17f8ae-ARN",
        },
        body: None,
    }
    ```
    - сделал protocol типы для (де)сериализации json сообщений как они приходят в WS
        - сделал тесты, чтобы проверить что все примеры json сообщений из документации десериализуются
        - сделал десериализацию Stream сообщений, data: serde_json::Value
    - сделал подключение по WS
- сделал тест который подписывается на канал "candles_minute_1"
    - и показывает несколько сообщений
    ```rust
    CandlesMessage { symbol: "BTC_USDT", amount: "90155.31551615", high: "96382.55", quantity: "0.935689", trade_count: 85, low: "96333.91", close_time: 1738867079999, start_time: 1738867020000, close: "96382.55", open: "96337.63", record_time: 1738867071361 }
    Kline { pair: "BTC_USDT", time_frame: "1m", o: 96337.63, h: 96382.55, l: 96333.91, c: 96382.55, utc_begin: 1738867020000, volume_bs: VBS { buy_base: 0.935689, sell_base: 0.0, buy_quote: 90155.31551615, sell_quote: 0.0 } }

    CandlesMessage { symbol: "BTC_USDT", amount: "94453.40158607", high: "96400.42", quantity: "0.980277", trade_count: 89, low: "96333.91", close_time: 1738867079999, start_time: 1738867020000, close: "96393.3", open: "96337.63", record_time: 1738867072361 }
    Kline { pair: "BTC_USDT", time_frame: "1m", o: 96337.63, h: 96400.42, l: 96333.91, c: 96393.3, utc_begin: 1738867020000, volume_bs: VBS { buy_base: 0.980277, sell_base: 0.0, buy_quote: 94453.40158607, sell_quote: 0.0 } }

    CandlesMessage { symbol: "BTC_USDT", amount: "95845.35222992", high: "96402.15", quantity: "0.994716", trade_count: 90, low: "96333.91", close_time: 1738867079999, start_time: 1738867020000, close: "96402.15", open: "96337.63", record_time: 1738867080120 }
    Kline { pair: "BTC_USDT", time_frame: "1m", o: 96337.63, h: 96402.15, l: 96333.91, c: 96402.15, utc_begin: 1738867020000, volume_bs: VBS { buy_base: 0.994716, sell_base: 0.0, buy_quote: 95845.35222992, sell_quote: 0.0 } }
    ```
- Особенности данных свечек:
    - По WS последняя свечка обновляется каждую секунду
    - т.е. свечка та же
    - start_time и close_time те же
    - а record_time и значения цен/объёма/кол-ва торгов меняется
    - получается что при сохранении в БД нужно не просто добавлять в конец новую запись
    - а искать нет ли уже такой по utc_begin
    - Получается следующая комбинацией полей KL должна быть уникальной:
        - pair
        - time_frame
        - utc_begin
- сделал тест который подписывается на канал "trades"
    - и показывает несколько сообщений
    ```rust
    TradesMessage { symbol: "BTC_USDT", amount: "1376.60073045", taker_side: Sell, quantity: "0.014349", create_time: 1738868627090, price: "95937.05", id: "121227976", record_time: 1738868627100 }
    RecentTrade { tid: "121227976", pair: "BTC_USDT", price: "95937.05", amount: "1376.60073045", side: "sell", timestamp: 1738868627090 }

    TradesMessage { symbol: "BTC_USDT", amount: "4.41309464", taker_side: Sell, quantity: "0.000046", create_time: 1738868627661, price: "95936.84", id: "121227977", record_time: 1738868627671 }
    RecentTrade { tid: "121227977", pair: "BTC_USDT", price: "95936.84", amount: "4.41309464", side: "sell", timestamp: 1738868627661 }

    TradesMessage { symbol: "BTC_USDT", amount: "1431.7645342", taker_side: Sell, quantity: "0.014924", create_time: 1738868627945, price: "95937.05", id: "121227978", record_time: 1738868627962 }
    RecentTrade { tid: "121227978", pair: "BTC_USDT", price: "95937.05", amount: "1431.7645342", side: "sell", timestamp: 1738868627945 }
    ```
    - тут вроде всё проще
    - или tid сам по себе уникальный
    - или комбинация tid+pair

- "все сообщения могут не поместиться в RAM"
    - "All data is maintained in memory (RAM), with data persistence ensured by write-ahead logging and snapshotting, and for those reasons some industry observers have compared Tarantool to Membase"
    - так а если сообщения в память приложения не поместятся, то они и в Tarantool получается не поместятся
    - если только Tarantool не крутится на отдельном сервере с большим количеством RAM
    - "In 2017 Tarantool introduced an optional on-disk storage engine which allows databases larger than memory size"
    - а, если так то ок.

- Для раста нет популярного поддерживаемого клиента для Tarantool
    - времени на разработку уже мало осталось
    - не хочу в последний момент узнать что ничего не работает и идти фиксить чужие крейты
    - возьму вместо tarantool другую БД
    - в реальном проекте не поздно заменить будет

- Возьму пока монгу
    - на типах уже есть #[derive(serde::Serialize)]
    - выйдет быстрее чем писать вручную через sqlx создавать таблицы и INSERTы
    - в ТЗ не указано какюу БД использовать
    - Tarantool заявлен как NoSql (хотя там есть SQL запросы)
        - значит монга ближе

- Завожу монгу на машине где раньше не заводил
    - Влючаю сервис
    ```
    systemctl enable mongodb
    sudo systemctl start mongodb
    mongo
    ```
    - Ругается что "Access control is not enabled for the database. Read and write access to data and configuration is unrestricted"
        - но мне так норм, порты закрыты, в БД ничего важно и после теста выключу
    - в `mongo`
    ```
    use bitsgap_qthree_test
    db.createUser({ user: "scraper", pwd: "scraper", roles: [ { role: "readWrite", db: "bitsgap_qthree_test" } ] })
    ```
    - запускаем scraper
    ```
    export API_KEY="$API_KEY"
    export SECRET_KEY="$SECRET_KEY"
    cargo run --release --bin bitsgap_scraper -- --mongodb-uri mongodb://scraper:scraper@localhost/bitsgap_qthree_test
    ```

- limit у `https://api.poloniex.com/markets/{symbol}/candles` работает не так как я думал
    - если указан startTime, а endTime не указан, то возвращает limit последних candles
    - т.е. какой бы startTime не указывай, он всё равно последние limit вернёт
    - может попробовать делать наоборот? указывать только endTime?

- Сделал выкачивание KL в обратном порядке
    - от последних 500, к предпоследним 500
    - с оганичением в 5000 на каждую пару symbol-interval:
    ```rust
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-06T14:36:00Z, end: 2025-02-06T22:55:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-06T06:16:00Z, end: 2025-02-06T14:35:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-05T21:56:00Z, end: 2025-02-06T06:15:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-05T13:36:00Z, end: 2025-02-05T21:55:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-05T05:16:00Z, end: 2025-02-05T13:35:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-04T20:56:00Z, end: 2025-02-05T05:15:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-04T12:36:00Z, end: 2025-02-04T20:55:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-04T04:16:00Z, end: 2025-02-04T12:35:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-03T19:56:00Z, end: 2025-02-04T04:15:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "1m", start: 2025-02-03T11:36:00Z, end: 2025-02-03T19:55:59.999Z
    Downloaded 500 klines, symbol: BTC_USDT, interval: "15m", start: 2025-02-01T18:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 143 klines, symbol: BTC_USDT, interval: "1h", start: 2025-02-01T00:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 6 klines, symbol: BTC_USDT, interval: "1d", start: 2025-02-01T00:00:00Z, end: 2025-02-06T23:59:59.999Z
    Downloaded 500 klines, symbol: TRX_USDT, interval: "1m", start: 2025-02-06T14:36:00Z, end: 2025-02-06T22:55:59.999Z
    Downloaded 500 klines, symbol: TRX_USDT, interval: "15m", start: 2025-02-01T18:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 143 klines, symbol: TRX_USDT, interval: "1h", start: 2025-02-01T00:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 6 klines, symbol: TRX_USDT, interval: "1d", start: 2025-02-01T00:00:00Z, end: 2025-02-06T23:59:59.999Z
    Downloaded 500 klines, symbol: ETH_USDT, interval: "1m", start: 2025-02-06T14:36:00Z, end: 2025-02-06T22:55:59.999Z
    Downloaded 500 klines, symbol: ETH_USDT, interval: "15m", start: 2025-02-01T18:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 143 klines, symbol: ETH_USDT, interval: "1h", start: 2025-02-01T00:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 6 klines, symbol: ETH_USDT, interval: "1d", start: 2025-02-01T00:00:00Z, end: 2025-02-06T23:59:59.999Z
    Downloaded 500 klines, symbol: DOGE_USDT, interval: "1m", start: 2025-02-06T14:36:00Z, end: 2025-02-06T22:55:59.999Z
    Downloaded 500 klines, symbol: DOGE_USDT, interval: "15m", start: 2025-02-01T18:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 143 klines, symbol: DOGE_USDT, interval: "1h", start: 2025-02-01T00:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 6 klines, symbol: DOGE_USDT, interval: "1d", start: 2025-02-01T00:00:00Z, end: 2025-02-06T23:59:59.999Z
    Downloaded 500 klines, symbol: BCH_USDT, interval: "1m", start: 2025-02-06T14:36:00Z, end: 2025-02-06T22:55:59.999Z
    Downloaded 500 klines, symbol: BCH_USDT, interval: "15m", start: 2025-02-01T18:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 143 klines, symbol: BCH_USDT, interval: "1h", start: 2025-02-01T00:00:00Z, end: 2025-02-06T22:59:59.999Z
    Downloaded 6 klines, symbol: BCH_USDT, interval: "1d", start: 2025-02-01T00:00:00Z, end: 2025-02-06T23:59:59.999Z
    Downloaded 10245 klines. Storage has 10245 klines.
    ```
    - Остаётся, конечно, вопрос, а нужны ли вообще 1-минутные KL 2-3 месячной давности...

- Получаем по WS новые KL и RT, сохраняем их в монгу
    - лог
    ```rust
    WS server sent event: Subscribe { channel: "candles_day_1" }
    WS server sent event: Subscribe { channel: "candles_hour_1" }
    WS server sent event: Subscribe { channel: "candles_minute_1" }
    WS server sent event: Subscribe { channel: "candles_minute_15" }
    WS server sent event: Subscribe { channel: "trades" }
    New recent trades: [RecentTrade { tid: "315898567", pair: "DOGE_USDT", price: "0.250016", amount: "27.469007904", side: "buy", timestamp: 1738888942023 }]
    New recent trades: [RecentTrade { tid: "315898568", pair: "DOGE_USDT", price: "0.250016", amount: "26.628954144", side: "buy", timestamp: 1738888942025 }]
    New recent trades: [RecentTrade { tid: "315898569", pair: "DOGE_USDT", price: "0.250016", amount: "27.469007904", side: "buy", timestamp: 1738888942025 }]
    New recent trades: [RecentTrade { tid: "315898570", pair: "DOGE_USDT", price: "0.250016", amount: "12.060521824", side: "buy", timestamp: 1738888942025 }]
    New recent trades: [RecentTrade { tid: "315898571", pair: "DOGE_USDT", price: "0.250016", amount: "27.469007904", side: "sell", timestamp: 1738888942036 }]
    New recent trades: [RecentTrade { tid: "315898572", pair: "DOGE_USDT", price: "0.250016", amount: "41.880930208", side: "buy", timestamp: 1738888942036 }]
    New recent trades: [RecentTrade { tid: "315898573", pair: "DOGE_USDT", price: "0.250016", amount: "57.6036864", side: "sell", timestamp: 1738888942043 }]
    New recent trades: [RecentTrade { tid: "315898574", pair: "DOGE_USDT", price: "0.250016", amount: "78.328512704", side: "sell", timestamp: 1738888942054 }]
    New recent trades: [RecentTrade { tid: "112284313", pair: "ETH_USDT", price: "2700.82", amount: "326.07540024", side: "sell", timestamp: 1738888942090 }]
    New recent trades: [RecentTrade { tid: "112284314", pair: "ETH_USDT", price: "2700.82", amount: "137.25837322", side: "buy", timestamp: 1738888942094 }]
    New recent trades: [RecentTrade { tid: "112284315", pair: "ETH_USDT", price: "2700.82", amount: "326.07540024", side: "sell", timestamp: 1738888942094 }]
    New recent trades: [RecentTrade { tid: "122249757", pair: "BCH_USDT", price: "318.6", amount: "49.0395492", side: "buy", timestamp: 1738888942156 }]
    New klines: [Kline { pair: "ETH_USDT", time_frame: "1m", o: 2700.27, h: 2702.28, l: 2699.95, c: 2700.82, utc_begin: 1738888920000, volume_bs: VBS { buy_base: 1.836918, sell_base: 0.0, buy_quote: 4961.77843053, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "TRX_USDT", time_frame: "1m", o: 0.23264, h: 0.23265, l: 0.23259, c: 0.23261, utc_begin: 1738888920000, volume_bs: VBS { buy_base: 30636.89, sell_base: 0.0, buy_quote: 7126.79129776, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "ETH_USDT", time_frame: "15m", o: 2709.45, h: 2715.03, l: 2695.19, c: 2700.82, utc_begin: 1738888200000, volume_bs: VBS { buy_base: 95.286661, sell_base: 0.0, buy_quote: 257711.64338665, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "TRX_USDT", time_frame: "15m", o: 0.23261, h: 0.2329, l: 0.2324, c: 0.23261, utc_begin: 1738888200000, volume_bs: VBS { buy_base: 277959.998, sell_base: 0.0, buy_quote: 64671.73773505, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "ETH_USDT", time_frame: "1h", o: 2687.93, h: 2716.37, l: 2686.03, c: 2700.82, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 268.514666, sell_base: 0.0, buy_quote: 726154.09120705, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "TRX_USDT", time_frame: "1h", o: 0.23164, h: 0.2329, l: 0.23162, c: 0.23261, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 2240773.507, sell_base: 0.0, buy_quote: 520661.81376926, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "ETH_USDT", time_frame: "1d", o: 2687.93, h: 2716.37, l: 2686.03, c: 2700.82, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 268.514666, sell_base: 0.0, buy_quote: 726154.09120705, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "TRX_USDT", time_frame: "1d", o: 0.23164, h: 0.2329, l: 0.23162, c: 0.23261, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 2240773.507, sell_base: 0.0, buy_quote: 520661.81376926, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "DOGE_USDT", time_frame: "1m", o: 0.250022, h: 0.250074, l: 0.249953, c: 0.250016, utc_begin: 1738888920000, volume_bs: VBS { buy_base: 36432.053, sell_base: 0.0, buy_quote: 9108.552903133, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "DOGE_USDT", time_frame: "15m", o: 0.250866, h: 0.251558, l: 0.249501, c: 0.250016, utc_begin: 1738888200000, volume_bs: VBS { buy_base: 1097531.519, sell_base: 0.0, buy_quote: 274902.377813602, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "DOGE_USDT", time_frame: "1h", o: 0.247898, h: 0.251848, l: 0.247574, c: 0.250016, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 4291415.952, sell_base: 0.0, buy_quote: 1074478.280878412, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "DOGE_USDT", time_frame: "1d", o: 0.247898, h: 0.251848, l: 0.247574, c: 0.250016, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 4291415.952, sell_base: 0.0, buy_quote: 1074478.280878412, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "BCH_USDT", time_frame: "1m", o: 318.25, h: 318.67, l: 318.25, c: 318.6, utc_begin: 1738888920000, volume_bs: VBS { buy_base: 6.512506, sell_base: 0.0, buy_quote: 2074.10257739, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "BCH_USDT", time_frame: "15m", o: 319.11, h: 319.73, l: 317.75, c: 318.6, utc_begin: 1738888200000, volume_bs: VBS { buy_base: 190.720458, sell_base: 0.0, buy_quote: 60777.14480149, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "BCH_USDT", time_frame: "1h", o: 316.07, h: 319.82, l: 315.75, c: 318.6, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 660.870036, sell_base: 0.0, buy_quote: 210535.05046351, sell_quote: 0.0 } }]
    New klines: [Kline { pair: "BCH_USDT", time_frame: "1d", o: 316.07, h: 319.82, l: 315.75, c: 318.6, utc_begin: 1738886400000, volume_bs: VBS { buy_base: 660.870036, sell_base: 0.0, buy_quote: 210535.05046351, sell_quote: 0.0 } }]
    ```
    - монга
    ```
    > show collections
    klines
    recent_trades
    > db.klines.count()
    10260
    > db.recent_trades.count()
    96
    
    > db.klines.find()
    { "_id" : ObjectId("67a556dc8668dda4a21b1117"), "pair" : "BTC_USDT", "time_frame" : "1m", "o" : 96972.75, "h" : 97018.23, "l" : 96940.59, "c" : 96964.21, "utc_begin" : NumberLong("1738858980000"), "volume_bs" : { "buy_base" : 0.561613, "sell_base" : 0.24809599999999998, "buy_quote" : 54462.99, "sell_quote" : 24059.799999999996 } }
    { "_id" : ObjectId("67a556dc8668dda4a21b1118"), "pair" : "BTC_USDT", "time_frame" : "1m", "o" : 96950.14, "h" : 96982.26, "l" : 96920.5, "c" : 96981.02, "utc_begin" : NumberLong("1738859040000"), "volume_bs" : { "buy_base" : 0.443164, "sell_base" : 0.40737999999999996, "buy_quote" : 42963.55, "sell_quote" : 39496.84999999999 } }
    { "_id" : ObjectId("67a556dc8668dda4a21b1119"), "pair" : "BTC_USDT", "time_frame" : "1m", "o" : 96979.23, "h" : 97056.76, "l" : 96979.23, "c" : 97037.8, "utc_begin" : NumberLong("1738859100000"), "volume_bs" : { "buy_base" : 0.333243, "sell_base" : 0.35223999999999994, "buy_quote" : 32325.71, "sell_quote" : 34169.950000000004 } }

    > db.recent_trades.find()
    { "_id" : ObjectId("67a556ee8668dda4a21b392b"), "tid" : "315898567", "pair" : "DOGE_USDT", "price" : "0.250016", "amount" : "27.469007904", "side" : "buy", "timestamp" : NumberLong("1738888942023") }
    { "_id" : ObjectId("67a556ee8668dda4a21b392c"), "tid" : "315898568", "pair" : "DOGE_USDT", "price" : "0.250016", "amount" : "26.628954144", "side" : "buy", "timestamp" : NumberLong("1738888942025") }
    { "_id" : ObjectId("67a556ee8668dda4a21b392d"), "tid" : "315898569", "pair" : "DOGE_USDT", "price" : "0.250016", "amount" : "27.469007904", "side" : "buy", "timestamp" : NumberLong("1738888942025") }
    ```
