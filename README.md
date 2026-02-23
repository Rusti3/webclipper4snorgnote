# notebooklm_runner

Проект принимает клипы из расширения `add4snorgnoteV3` и сохраняет их как локальные Markdown-заметки.

## Саммари версии 0.3.0
- Полностью убран пользовательский поток NotebookLM (`start.txt/end.txt`, sidecar Playwright и связанные команды).
- Добавлена поддержка deep-link формата расширения v3:
  - `snorgnote://new?clipId=<uuid>&source=web-clipper`
- Добавлен клиент helper API:
  - `GET /clips/:clipId`
  - `DELETE /clips/:clipId`
  - `GET /health`
- Добавлено сохранение клипов в `./notes/*.md` с метаданными.
- После успешного сохранения заметки клип удаляется из helper (`DELETE /clips/:clipId`).
- Добавлены новые тесты deep-link/helper/writer/full-flow.

## Что изменилось для пользователя
- Из расширения `add4snorgnoteV3` теперь можно открыть приложение deep-link'ом, и заметка создастся автоматически.
- Не нужно вручную копировать содержимое страницы или выделение.

## Протокол интеграции с add4snorgnoteV3

### Расширение отправляет deep-link
`snorgnote://new?clipId=<uuid>&source=web-clipper`

### Приложение обращается к helper
- `GET http://127.0.0.1:27124/clips/<clipId>`
- `DELETE http://127.0.0.1:27124/clips/<clipId>` после успешного сохранения

### Формат payload helper
```json
{
  "type": "full_page",
  "title": "Page title",
  "url": "https://example.com",
  "contentMarkdown": "markdown content",
  "createdAt": "2026-02-24T00:00:00.000Z"
}
```

## Запуск

### 1) Запустить helper из проекта расширения
```powershell
cd D:\Snorgnote\add4snorgnoteV3
npm install
npm run helper:start
```

### 2) Проверить helper из нашего приложения
```powershell
cd D:\add4snorgnote
cargo run -- helper-health
```

### 3) Обработать deep-link
```powershell
cargo run -- deeplink "snorgnote://new?clipId=<uuid>&source=web-clipper"
```

Параметры:
- `--notes-dir` (по умолчанию `notes`)
- `--helper-base-url` (по умолчанию `http://127.0.0.1:27124`)
- `--timeout-sec` (по умолчанию `15`)

## Где лежат заметки
- По умолчанию: `./notes`
- Формат имени: `YYYYMMDD-HHMMSS-<slug-title>-<clipId8>.md`

## Тесты
```powershell
cargo test
```

Текущий набор:
- `tests/deeplink_new_tests.rs`
- `tests/helper_client_tests.rs`
- `tests/note_writer_tests.rs`
- `tests/clipper_flow_tests.rs`
