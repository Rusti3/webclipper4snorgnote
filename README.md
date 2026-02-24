# notebooklm_runner

## Саммари версии 0.4.6
- Для кнопок `Save full page` и `Save selected text` в popup запуск `snorgnote://...` теперь делается из текущей активной страницы через `content script`.
- Подтверждение браузера (`Сайт ... собирается открыть это приложение`) теперь показывается на той странице, с которой делается клиппинг, а не в окне расширения.
- Popup больше не перенаправляет сам себя на deep-link, а показывает статус запуска из текущей страницы.
- Добавлена проверка валидности deep-link в `content.js` с явной ошибкой для невалидных URL.
- Обновлены тесты (`background/content/popup`) под новый поток запуска, все Node-тесты проходят.

## Саммари версии 0.4.5
- Изменён источник запуска deep-link: теперь запуск делается из extension-контекста (`chrome-extension://...`), а не из страницы сайта.
- Для `popup`:
  - после успешного capture popup сам открывает `snorgnote://...`.
- Для `context menu`:
  - background автоматически открывает `launcher.html` (страница расширения),
  - launcher запускает deep-link и пробует автозакрыться через ~1.2 секунды.
- Удалён запуск `open_deeplink` из `content.js`, чтобы браузер не показывал источник как сайт.
- Добавлены Node-тесты:
  - `tests/extension.popup.test.js`
  - `tests/extension.launcher.test.js`

Проект принимает клипы из browser extension и сохраняет их в Markdown-заметки.
## Саммари версии 0.4.4
- Изменён способ запуска deep-link из расширения:
  - больше не открывается новая вкладка `snorgnote://...` через `chrome.tabs.create`;
  - запуск приложения выполняется в контексте текущей вкладки через content script.
- При отправке клипа пользователь остаётся на той же странице без дополнительного перехода.
- Добавлена обработка `open_deeplink` в `content.js` с проверкой валидности URL (`snorgnote://`).
- Для страниц, где запуск недоступен, показывается ошибка без fallback-открытия новой вкладки.
- Добавлены Node-тесты:
  - `tests/extension.background.test.js`
  - `tests/extension.content.test.js`

## Саммари версии 0.4.3
- Откатено изменение версии 0.4.2 по выбору вкладки в extension.
- Возвращено прежнее поведение:
  - клиппинг из popup работает только для текущей активной `http/https` вкладки.
- Удалены файлы откатанного изменения:
  - `src/extension/tab.js`
  - `tests/extension.tab.test.js`

## Саммари версии 0.4.1
- Добавлена авто-регистрация deep-link протокола `snorgnote://` при старте приложения на Windows.
- Регистрация выполняется в `HKCU\Software\Classes\snorgnote` (без админ-прав).
- Если протокол уже настроен, повторной записи нет.
- Если путь к `exe` изменился, запись автоматически обновляется.
- Добавлена команда `install-protocol` для ручной проверки/переустановки регистрации.
- Добавлен тест `tests/protocol_registration_tests.rs` на корректный формат команды запуска.

## Саммари версии 0.4.0
- Убран helper API из пользовательского потока.
- Расширение теперь передаёт контент напрямую в deep-link:
  - `snorgnote://new?data=<base64url(json)>`
- Приложение декодирует payload, валидирует поля и сохраняет заметку в `./notes`.
- Добавлена защита от слишком больших payload:
  - в расширении контент обрезается перед кодированием deep-link,
  - в приложении есть повторная обрезка и пометка `[CLIPPED: ...]`.
- Удалены команды/код helper-health и HTTP-клиента helper.
- Добавлены тесты нового direct-потока (Rust + Node).

## Что изменилось для пользователя
- Не нужно запускать helper вручную.
- После первого запуска приложения deep-link протокол регистрируется автоматически.
- Расширение может отправлять выделение и страницу сразу в приложение через deep-link.
- Заметка автоматически появляется в папке `notes`.

## Текущий протокол

### Deep-link
`snorgnote://new?data=<base64url(json)>`

### Payload
```json
{
  "type": "full_page",
  "title": "Page title",
  "url": "https://example.com/page",
  "contentMarkdown": "markdown content",
  "createdAt": "2026-02-24T00:00:00.000Z",
  "source": "web-clipper"
}
```

## Запуск

### 1) Проверка тестов
```powershell
cd D:\add4snorgnote
cargo test
npm test
```

### 2) Первый запуск (авто-регистрация протокола)
```powershell
cargo run --
```

### 3) Ручная переустановка регистрации протокола (опционально)
```powershell
cargo run -- install-protocol
```

### 4) Ручной запуск обработчика deep-link
```powershell
cargo run -- deeplink "snorgnote://new?data=<...>"
```

Параметры:
- `--notes-dir` (по умолчанию `notes`)
- `--timeout-sec` (по умолчанию `15`)

## Заметки
- Путь по умолчанию: `./notes`
- Формат имени файла: `YYYYMMDD-HHMMSS-<slug-title>-<clipId8>.md`

## Extension (в этом же репозитории)
- `manifest.json`
- `src/extension/background.js`
- `src/extension/content.js`
- `src/extension/launcher.html`
- `src/extension/launcher.js`
- `src/extension/payload.js`
- `src/extension/popup.*`

## Тесты

### Rust
- `tests/deeplink_new_tests.rs`
- `tests/clipper_direct_flow_tests.rs`
- `tests/note_writer_tests.rs`
- `tests/protocol_registration_tests.rs`

### Node
- `tests/extension.payload.test.js`
- `tests/extension.background.test.js`
- `tests/extension.content.test.js`
- `tests/extension.popup.test.js`
- `tests/extension.launcher.test.js`

