# PROGRESS.md — doc-format project

**Обновлено:** 7 апреля 2026

---

## Текущая фаза: Phase 1 — Go basics + format syntax design

**Статус:** не начата

---

## Стек

- Язык: Rust
- Режим: Claude Code пишет код, Artem архитектурит и ревьюит

---

## Format specification — прогресс

- [x] Базовый синтаксис определён на бумаге
- [ ] Типы блоков: heading, paragraph, table, figure, code
- [x] Layout: grid-based (не coordinate-based)
- [x] Block ID схема
- [x] Certificate схема
- [x] Asset handling: inline (base64) vs linked (hash)

### Синтаксис (черновик)

```
STYLES({
  #mainColor: #FF0000
  #mainFont: "Arial"
})

PAGE(
  STYLES({
    #bgColor: #FFFFFF
  })

  H1({color: #mainColor} Hello World)

  P(Текст параграфа)

  GRID({columns: "1fr 2fr"}
    P(Левая колонка)
    P(Правая колонка)
  )
)
```

Правила:

- Блок: `TYPE({атрибуты} контент)` или `TYPE(контент)`
- Атрибуты опциональны
- STYLES всегда первый блок (документ и страница)
- Переменные: `#name`, доступны везде (два прохода парсера)
- Grid: columns = фиксированные / пропорции (fr) / auto

---

## Renderer — прогресс

- [x] AST → JSON
- [x] AST → plain text

## Lexer — прогресс

- [x] Определены токены
- [x] Написан базовый Lexer (mode-based: Normal / Attrs / Content)
- [x] Тесты для всех типов токенов

## Parser — прогресс

- [x] AST определён (Document, Block, Content, Value)
- [x] Базовый Parser: токены → AST
- [x] Переменные: подстановка #var в атрибутах (два прохода)
- [x] Тесты

---

## Открытые вопросы

_(нет)_

---

## Решения принятые

- Язык реализации: Go (простота для контрибьюторов, скорость парсинга, один бинарник)
- Семантические блоки вместо визуальных координат
- Diff-friendly: стабильные block ID
- Два режима ассетов: self-contained (base64) и linked (external + hash)
- Верификация без центрального CA — самодостаточная
- Синтаксис формата: human-readable текстовый (не JSON/YAML)
- Sparse layout: координаты в абсолютных единицах (mm)
- Certificate: SHA-256 хеш всего документа
- Folio = формат хранения; редактор и authoring syntax — отдельные проекты поверх
