#!/usr/bin/env python3

import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:
    try:
        import tomli as tomllib
    except ModuleNotFoundError:
        print("Error: Python 3.11+ (tomllib) or the 'tomli' package is required.")
        sys.exit(1)

LOCALES_DIR = Path("crates/anime-games-launcher/assets/locales")


def load_all_locales():
    """Load all .toml files from the locales directory into a dict keyed by filename stem."""
    locales = {}
    if not LOCALES_DIR.is_dir():
        print(f"Error: locales directory not found at '{LOCALES_DIR}'")
        sys.exit(1)
    for toml_file in sorted(LOCALES_DIR.glob("*.toml")):
        with open(toml_file, "rb") as f:
            data = tomllib.load(f)
        rel_path = f"./{toml_file}"
        locales[toml_file.stem] = {"path": rel_path, "data": data}
    return locales


def format_toml_file(filepath):
    """
    Parses a TOML file preserving comments and blank lines, sorts the language
    keys within each section based on priority and variants, and saves it.
    Returns a tuple of (was_changed, num_keys).
    """
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')

    header_lines = []
    sections = []
    current_section = None
    current_prefix = []

    for line in lines:
        stripped = line.strip()

        if stripped.startswith('[') and stripped.endswith(']'):
            if current_section is not None:
                current_section['suffix'] = current_prefix
                sections.append(current_section)
            else:
                header_lines.extend(current_prefix)

            current_prefix = []
            current_section = {
                'header': line,
                'entries': [],
                'suffix': []
            }
        elif current_section is not None:
            if '=' in stripped and not stripped.startswith('#'):
                lang = stripped.split('=', 1)[0].strip()
                current_section['entries'].append({
                    'prefix': current_prefix,
                    'lang': lang,
                    'line': line
                })
                current_prefix = []
            else:
                current_prefix.append(line)
        else:
            current_prefix.append(line)

    if current_section is not None:
        current_section['suffix'] = current_prefix
        sections.append(current_section)
    elif current_prefix:
        header_lines.extend(current_prefix)

    def sort_key(lang):
        base = lang[:2]
        priority = {
            'en': 0,
            'ru': 1,
            'es': 2,
            'pt': 3,
            'fr': 4,
            'de': 5
        }.get(base, 6)
        is_specific = 0 if lang == base else 1
        return (priority, base, is_specific, lang)

    output_lines = []
    output_lines.extend(header_lines)

    for section in sections:
        output_lines.append(section['header'])

        section['entries'].sort(key=lambda e: sort_key(e['lang']))

        for entry in section['entries']:
            output_lines.extend(entry['prefix'])
            output_lines.append(entry['line'])

        output_lines.extend(section['suffix'])

    new_content = '\n'.join(output_lines)

    if content.endswith('\n') and not new_content.endswith('\n'):
        new_content += '\n'

    changed = new_content != content
    if changed:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)

    return changed, len(sections)


def cmd_list():
    locales = load_all_locales()
    if not locales:
        print("No locale files found.")
        return

    max_name_len = max(len(name) for name in locales) if locales else 0
    for file_name, file_info in locales.items():
        print(f"{file_name:<{max_name_len}}  ({file_info['path']})")


def cmd_tree():
    locales = load_all_locales()
    if not locales:
        print("No locale files found.")
        return

    for file_name, file_info in locales.items():
        data = file_info["data"]
        rel_path = file_info["path"]

        sections = {k: v for k, v in data.items() if isinstance(v, dict)}
        if not sections:
            print(f"{file_name} ({rel_path})")
            print()
            continue

        max_key_len = max(len(k) for k in sections)

        all_langs = sorted(set(lang for trans in sections.values() for lang in trans))
        lang_widths = {lang: len(lang) for lang in all_langs}

        print(f"{file_name} ({rel_path})")
        for key, translations in sections.items():
            line = f"  {key:<{max_key_len}}  "

            for lang in all_langs:
                width = lang_widths[lang]
                if lang in translations:
                    line += f"{lang:<{width}} "
                else:
                    line += f"{' ' * width} "

            print(line.rstrip())
        print()


def cmd_available(locale_key):
    locales = load_all_locales()
    found_any = False

    for file_name, file_info in locales.items():
        data = file_info["data"]
        rel_path = file_info["path"]

        sections = {k: v for k, v in data.items() if isinstance(v, dict)}
        matching = {k: v for k, v in sections.items() if locale_key in v}

        if matching:
            found_any = True
            print(f"{file_name} ({rel_path})")
            max_key_len = max(len(k) for k in matching)
            for key, translations in matching.items():
                value = translations[locale_key]
                print(f"  {key:<{max_key_len}}  {value}")
            print()

    if not found_any:
        print(f"No translations found for locale '{locale_key}'.")


def cmd_missing(locale_key=None):
    locales = load_all_locales()
    found_any = False

    if locale_key:
        for file_name, file_info in locales.items():
            data = file_info["data"]
            rel_path = file_info["path"]

            sections = {k: v for k, v in data.items() if isinstance(v, dict)}
            missing = {k: v for k, v in sections.items() if locale_key not in v}

            if missing:
                found_any = True
                print(f"{file_name} ({rel_path})")
                max_key_len = max(len(k) for k in missing)
                for key, translations in missing.items():
                    available = sorted(translations.keys())
                    avail_str = ", ".join(available) if available else "none"
                    print(f"  {key:<{max_key_len}}  missing  (has: {avail_str})")
                print()

        if not found_any:
            print(f"No missing translations for locale '{locale_key}' — all keys have it!")

    else:
        all_global_langs = set()
        for file_info in locales.values():
            data = file_info["data"]
            sections = {k: v for k, v in data.items() if isinstance(v, dict)}
            for trans in sections.values():
                all_global_langs.update(trans.keys())

        all_global_langs = sorted(all_global_langs)

        if not all_global_langs:
            print("No languages found in any locale files.")
            return

        for file_name, file_info in locales.items():
            data = file_info["data"]
            rel_path = file_info["path"]

            sections = {k: v for k, v in data.items() if isinstance(v, dict)}
            if not sections:
                continue

            missing_in_file = {}
            for key, translations in sections.items():
                missing_langs = [lang for lang in all_global_langs if lang not in translations]
                if missing_langs:
                    missing_in_file[key] = missing_langs

            if missing_in_file:
                found_any = True
                print(f"{file_name} ({rel_path})")
                max_key_len = max(len(k) for k in missing_in_file)
                for key, missing_langs in missing_in_file.items():
                    missing_str = " ".join(missing_langs)
                    print(f"  {key:<{max_key_len}}  {missing_str}")
                print()

        if not found_any:
            print("No missing translations — all keys have translations for all available languages!")


def cmd_format(files_to_format=None):
    locales = load_all_locales()
    if not locales:
        print("No locale files found.")
        return

    if files_to_format:
        valid_files = {}
        for f in files_to_format:
            if f in locales:
                valid_files[f] = locales[f]
            else:
                print(f"Warning: File '{f}.toml' not found in {LOCALES_DIR}")

        if not valid_files:
            print("No valid files specified for formatting.")
            return

        files_to_process = valid_files
    else:
        files_to_process = locales

    files_changed = 0
    total_keys = 0

    for file_name, file_info in files_to_process.items():
        filepath = Path(file_info['path'].lstrip('./'))
        changed, keys = format_toml_file(filepath)
        total_keys += keys
        if changed:
            files_changed += 1
            print(f"  Updated {file_info['path']}")

    print(f"\nFormatted {len(files_to_process)} files ({files_changed} changed), containing {total_keys} translation keys.")


def cmd_help():
    print(
        "Usage: locales.py <command> [arguments]\n"
        "\n"
        "Commands:\n"
        "  list                  List all available locale files and their paths\n"
        "  tree                  Show a tree of all locale files, keys, and available languages\n"
        "  available <locale>    List keys that have a translation in the given language\n"
        "  missing [locale]      With <locale>: list keys missing that specific language.\n"
        "                        Without <locale>: list keys missing any available language.\n"
        "  format [files...]     Format and sort locale files by language priority.\n"
        "                        If no files provided, formats all files.\n"
        "  help                  Show this help message\n"
        "\n"
        "Examples:\n"
        "  python3 locales.py list\n"
        "  python3 locales.py tree\n"
        "  python3 locales.py available de\n"
        "  python3 locales.py missing pt\n"
        "  python3 locales.py missing\n"
        "  python3 locales.py format interface errors\n"
        "  python3 locales.py format"
    )


def main():
    args = sys.argv[1:]

    if len(args) == 0 or args[0] == "help":
        cmd_help()
    elif args[0] == "list":
        cmd_list()
    elif args[0] == "tree":
        cmd_tree()
    elif args[0] == "available":
        if len(args) < 2:
            print("Error: 'available' requires a locale key (e.g., 'en', 'de').")
            sys.exit(1)
        cmd_available(args[1])
    elif args[0] == "missing":
        if len(args) > 1:
            cmd_missing(args[1])
        else:
            cmd_missing()
    elif args[0] == "format":
        cmd_format(args[1:])
    else:
        print(f"Unknown command: '{args[0]}'\n")
        cmd_help()
        sys.exit(1)


if __name__ == "__main__":
    main()
