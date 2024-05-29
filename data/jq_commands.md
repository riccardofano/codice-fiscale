# JQ command to run to extract the correct data

Input files come from here: https://www.anagrafenazionale.interno.it/area-tecnica/tabelle-di-decodifica/
They need to be converted to JSON first.

"Stati Esteri" and "Archivio Comuni" are the ones you need.

## Queries

```bash
# Active
.[] | select(.active) | {code, province, name: .name_slugs[]} | \"\(.code),\(.name),\(.province)\"

# Inactive
.[] | select(.active | not) | {code, province, name: .name_slugs[]} | \"\(.code),\(.name),\(.province)\"
```

## Running

On MacOS / Linux

```bash
jq -r [query] [input-files] > [output-file]
```

On Windows Powershell

```powershell
jq -r [query] [input-files] | Out-File -Encoding "ASCII" [output-file]
```
