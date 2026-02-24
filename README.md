# notebooklm_runner

Проект принимает клипы из browser extension и сохраняет их в Markdown-заметки.

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

### 2) Ручной запуск обработчика deep-link
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
- `src/extension/payload.js`
- `src/extension/popup.*`

## Тесты

### Rust
- `tests/deeplink_new_tests.rs`
- `tests/clipper_direct_flow_tests.rs`
- `tests/note_writer_tests.rs`

### Node
- `tests/extension.payload.test.js`
