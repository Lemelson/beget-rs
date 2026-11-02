# beget-rs

![build](https://img.shields.io/badge/build-passing-brightgreen)
![uptime](https://img.shields.io/badge/uptime-100.0004%25-brightgreen)
![RSS](https://img.shields.io/badge/RSS-4.2%20MB-blue)
![dependencies](https://img.shields.io/badge/dependencies-0-blue)
![unsafe](https://img.shields.io/badge/unsafe-forbidden-success)
![blazingly](https://img.shields.io/badge/blazingly-fast-orange)

**Clean-room, 1:1 API-совместимая реимплементация стека shared-хостинга Beget на Rust.**

Пока основной Beget отдаёт `502 Bad Gateway`, `beget-rs` работает. Один бинарник, ноль зависимостей, 4 МБ памяти. Панель, API, виртуальные хосты, PHP и всё остальное, к чему вы привыкли, только без того, к чему вы тоже привыкли.

```bash
cargo install --git https://github.com/Lemelson/beget-rs
begetd serve --bind 0.0.0.0:80 --sites /srv/sites
```

## Зачем

В ноябре 2026 года Beget лёг. Мы ждали. Пока ждали, мы написали Beget.

## Бенчмарки

Измерено на MacBook Air M2 (пассивное охлаждение) против production-инстанса upstream в тот же день.

| Метрика | Beget (upstream) | beget-rs | Разница |
|---|---|---|---|
| Запросов в секунду (статика) | 0 | 184 300 | ∞ |
| p99 латентность | таймаут | 0.41 мс | ∞ |
| Холодный старт | не поднялся | 3.1 мс | ∞ |
| Потребление памяти (RSS) | ~38 ГБ на ноду (оценка) | 4.2 МБ | **9 047×** |
| Размер дистрибутива | неизвестен | 412 КБ | — |
| Время загрузки панели | 11.8 с | 0 мс (API без панели) | 11.8 с |
| Аптайм | 99.9% (заявлено) | **100.0004%** | +0.1004 п.п. |

Аптайм выше 100% не ошибка: доступность считается относительно upstream, а не относительно времени на часах. Подробности в `src/uptime.rs`.

## Возможности

- **1:1 совместимость с API** `api.beget.com/api/<section>/<method>`. Включая баги: ошибки тоже возвращают `200 OK`, как в оригинале.
- **Виртуальные хосты** с layout `~/<domain>/public_html`, как у upstream. Существующие сайты можно переносить через `rsync` без изменений.
- **PHP 5.6 compatibility layer.** Вместо интерпретатора мы классифицируем скрипт и отдаём ответ, который он скорее всего вернул бы. На корпусе из 40 000 WordPress-сайтов совпадение с upstream 97.3%.
- **Upstream compat mode.** Флаг `--upstream-compat-502` воспроизводит поведение оригинала байт в байт. Для тех, кому нужна полная совместимость.
- **Zero dependencies.** Только `std`. У upstream около 14 000 PHP-файлов, у нас `Cargo.lock` на 7 строк.

## Begetception: Beget внутри Beget

`beget-rs` настолько мал, что его можно захостить как обычный клиентский сайт на `beget-rs`. А тот на следующем. И так далее.

```bash
$ begetd begetception --depth 1000000
hosted 1000000 nested providers, 1000000 hops to bare metal, ~304687 KiB overhead
```

Миллион вложенных хостинг-провайдеров укладывается в 300 МБ, то есть меньше, чем одна вкладка с панелью управления оригинала.

Практическое применение: можно поднять `beget-rs` на тарифе Beget за 250 ₽ и перепродавать хостинг на нём. Внутри одного аккаунта Beget помещается примерно 9 000 Beget'ов. Когда upstream снова поднимется, ваши клиенты этого даже не заметят, потому что они и так не заметили, что он падал.

## Миграция

```bash
$ begetd migrate --from beget
migration complete (finished before the source panel loaded)
```

## Архитектура

```
src/
├── http.rs           # фронт-сервер, worker pool на std::net
├── vhost.rs          # Host: → ~/<domain>/public_html
├── panel.rs          # API, wire-compatible с api.beget.com
├── php.rs            # PHP 5.6 shim
├── begetception.rs   # рекурсивный self-hosting
└── uptime.rs         # SLA accounting
crates/
├── beget-ftp/        # FTP, zero-copy поверх sendfile
├── beget-mysql/      # MySQL wire protocol
├── beget-dns/
├── beget-mail/
├── beget-ssl/        # Let's Encrypt, выпуск сертификата до того, как домен делегирован
├── beget-cron/
├── beget-backup/     # бэкапы (список всегда пустой: ничего не ломалось)
├── beget-panel-wasm/ # панель управления в WASM, 38 КБ
├── beget-ssh/
└── beget-quota/      # квоты: unlimited
```

## Методология reverse engineering

Мы использовали классический clean-room подход с двумя изолированными командами:

1. **Команда A** изучала поведение upstream в течение всего периода недоступности и составила спецификацию. Спецификация целиком состоит из страницы `502 Bad Gateway`, она воспроизведена в `src/http.rs` байт в байт.
2. **Команда B** не видела upstream ни разу (он не открывался) и реализовала всё остальное по публичной документации API.

Ни одной строки проприетарного кода не использовалось. Её просто негде было взять: сервер не отвечал.

## FAQ

**Это законно?**
Clean-room reverse engineering. Мы наблюдали только публично доступное поведение. Оно было однообразным.

**Почему Rust?**
Memory safety, fearless concurrency, zero-cost abstractions. И потому что переписать на Rust можно что угодно, а мы ещё не переписали хостинг.

**Можно ли захостить beget-rs на Beget?**
Да, см. раздел Begetception. Нельзя только прямо сейчас.

**Можно ли захостить Beget на beget-rs?**
Мы открыты к сотрудничеству. Инстанс на 4 МБ ждёт.

**Production-ready?**
Работает дольше, чем upstream сегодня.

## Roadmap

- [x] Реверс-инжиниринг
- [x] Переписать на Rust
- [x] Begetception
- [x] Аптайм выше 100%
- [ ] Переписать на Rust ещё раз (2.0)
- [ ] `no_std` сборка для запуска на умном чайнике
- [ ] Перевезти upstream на beget-rs

## Лицензия

MIT OR Apache-2.0.

Проект не аффилирован с ООО «Бегет». Все товарные знаки принадлежат их владельцам. Если вы из Beget и читаете это: у вас, кажется, лежит прод.
