# JQ command to run to extract the correct data

Input files come from here: https://www.anagrafenazionale.interno.it/area-tecnica/tabelle-di-decodifica/
They need to be converted to JSON first.

Stati Esteri and Archivio Comuni

```bash
cat [input-file] | jq 'map(.name_slugs[] as $slug | {(($slug + \"|\" + .province) | ascii_upcase): {active, code}}) | add' > [output-file]
```

From Windows

```powershell
cat [input-file] | jq 'map(.name_slugs[] as $slug | {(($slug + \"|\" + .province) | ascii_upcase): {active, code}}) | add' | Out-File -Encoding "ASCII" [output-file]
```
