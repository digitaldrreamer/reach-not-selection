"""Build reach-not-selection-full.pdf from the Markdown source.
Requires: python-markdown, weasyprint. Run from papers/."""
import re, markdown, subprocess, pathlib
src = pathlib.Path("reach-not-selection-full.md").read_text()
fm, body = re.match(r"---\n(.*?)\n---\n(.*)", src, re.S).groups()
meta = dict(re.findall(r'^(\w+):\s*"(.*)"$', fm, re.M))
html_body = markdown.markdown(body, extensions=["tables"])
# Figures: turn <p><img alt="Figure N. caption"></p> into figure + caption
html_body = re.sub(r'<p><img alt="([^"]*)" src="([^"]*)" /></p>',
                   r'<figure><img src="\2" alt="\1"/><figcaption>\1</figcaption></figure>', html_body)
css = """
@page { size: A4; margin: 22mm 20mm; @bottom-center { content: counter(page); font-size: 9pt; color: #555; } }
body { font-family: "Charter", "Georgia", serif; font-size: 10.5pt; line-height: 1.45; color: #1b1f24; }
h1.title { font-size: 20pt; margin: 0 0 4pt; line-height: 1.2; }
p.sub { font-size: 11pt; color: #444; margin: 0 0 2pt; }
p.meta { font-size: 10pt; color: #444; margin: 0 0 14pt; }
h2 { font-size: 13pt; margin: 16pt 0 6pt; } h3 { font-size: 11pt; margin: 12pt 0 4pt; }
table { border-collapse: collapse; margin: 8pt 0; font-size: 9.5pt; width: 100%; }
th, td { border-top: 0.5pt solid #bbb; border-bottom: 0.5pt solid #bbb; padding: 3pt 6pt; text-align: left; vertical-align: top; }
th { border-top: 1pt solid #333; border-bottom: 1pt solid #333; }
figure { margin: 10pt 0; page-break-inside: avoid; } figure img { width: 100%; }
figcaption { font-size: 9pt; color: #333; margin-top: 4pt; }
code { font-size: 9pt; } ul, ol { margin: 4pt 0 4pt 16pt; padding: 0; } li { margin: 2pt 0; }
"""
html = f"""<!doctype html><html><head><meta charset="utf-8"><title>{meta['title']}</title><style>{css}</style></head><body>
<h1 class="title">{meta['title']}</h1><p class="sub">{meta['subtitle']}</p><p class="meta">{meta['author']} · {meta['date']}</p>
{html_body}</body></html>"""
pathlib.Path("reach-not-selection-full.html").write_text(html)
subprocess.run(["weasyprint", "reach-not-selection-full.html", "reach-not-selection-full.pdf"], check=True)
print("built reach-not-selection-full.pdf")
