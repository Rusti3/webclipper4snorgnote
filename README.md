# notebooklm_runner

CLI-приложение для сценария:

`start.txt -> NotebookLM -> end.txt`

С поддержкой deep-link входа от web clipper:

`snorgnote://clip?data=<base64url(json)>`

## Саммари версии 0.1.0
- Реализован one-shot CLI сценарий `start.txt -> NotebookLM -> end.txt`.
- Добавлен Rust-оркестратор с командами sidecar:
  - `connect`
  - `create_notebook`
  - `import_urls`
  - `ask`
  - `close`
- Добавлен Node.js sidecar на Playwright с persistent профилем.
- Добавлено логирование в `logs/notebooklm-YYYYMMDD.log`.
- Добавлены unit и mock e2e тесты.

## Саммари версии 0.2.0
- Добавлен deep-link вход:
  - новая CLI команда `deeplink`
  - формат: `snorgnote://clip?data=<base64url(json)>`
- Добавлен модуль `src/deeplink.rs`:
  - парсинг deep-link URI
  - декодирование `data` (base64url + JSON)
  - fallback-режим query-параметров (`prompt`, `url`)
  - валидация payload (prompt + минимум 1 валидный URL)
  - запись `start.txt` из payload
- Добавлена функция `run_from_deeplink(...)`:
  - принимает deep-link
  - генерирует `start.txt`
  - запускает существующий pipeline до `end.txt`
- Если в payload есть `title`, он приоритетнее CLI `--title`.
- Добавлены тесты deep-link:
  - `tests/deeplink_tests.rs`
  - `tests/deeplink_flow.rs`

## Установка
1. Установить зависимости sidecar:

```powershell
cd sidecar
npm install
cd ..
```

2. Проверить тесты:

```powershell
cargo test
```

## Запуск обычного сценария
```powershell
cargo run -- run --input start.txt --output end.txt --title "Auto Notebook"
```

## Формат start.txt
- Первая непустая строка: `PROMPT=...`
- Далее: URL по одному в строке.
- Пустые строки и строки с `#` игнорируются.

Пример:

```txt
PROMPT=Сделай краткое резюме источников и 5 ключевых тезисов.
https://example.com/article
https://youtu.be/example
```

## Deep-link режим
Запуск через CLI:

```powershell
cargo run -- deeplink "snorgnote://clip?data=..." --input start.txt --output end.txt
```

При вызове:
1. URI декодируется в payload.
2. Из payload записывается `start.txt`.
3. Выполняется pipeline NotebookLM.
4. Ответ пишется в `end.txt`.

## Контракт payload для web clipper
JSON внутри `data`:

```json
{
  "prompt": "Сделай резюме и 5 тезисов",
  "urls": [
    "https://example.com/a",
    "https://example.com/b"
  ],
  "source": "web-clipper",
  "title": "Notebook from clipper"
}
```

Поля:
- `prompt` (обязательно)
- `urls` (обязательно, минимум 1 URL)
- `source` (опционально)
- `title` (опционально; переопределяет `--title`)

## Что добавить в проект web clipper (Chrome Extension MV3)
### 1. Функция base64url + открытие deep-link
```js
function toBase64UrlUtf8(obj) {
  const json = JSON.stringify(obj);
  const bytes = new TextEncoder().encode(json);
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

async function sendToSnorgnote({ prompt, urls, title }) {
  const payload = {
    prompt,
    urls,
    source: "web-clipper",
    title
  };
  const data = toBase64UrlUtf8(payload);
  const deeplink = `snorgnote://clip?data=${data}`;
  await chrome.tabs.create({ url: deeplink });
}
```

### 2. Где вызывать
- В обработчике кнопки “Send to Snorgnote”.
- Перед вызовом отфильтровать пустые и дублирующиеся URL.

### 3. Важный момент по ОС
Чтобы deep-link реально открывал приложение, в системе должен быть зарегистрирован обработчик схемы `snorgnote://`.

Для desktop-приложения на Tauri это обычно делается в конфиге приложения/инсталлере.  
Для текущего CLI можно настроить handler через системную регистрацию протокола (Windows registry / installer).

## Формат end.txt
Секции:
- `Status`
- `Notebook`
- `Prompt`
- `Imported`
- `Answer`
- `Errors`
- `Timing`

## Live тест (ручной)
```powershell
cargo test -- --ignored
```
