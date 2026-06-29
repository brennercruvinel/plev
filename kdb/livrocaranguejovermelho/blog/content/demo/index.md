+++
title = "demo page"
[extra]
go_to_top = true
styles = ["demo/demo.css"]
scripts = ["demo/demo.js"]
katex = true
archive = "this page is in fact not archived. it is only here to demonstrate the archival statement."
trigger = "this page contains blackjack and hookers, and bad jokes such as this one."
disclaimer = """
- all tricks in this page are performed by the lab boys, don't try this at home.
- don't expose yourself to 4000° kelvin.
- don't take party escort submission position.
- don't interact with asbestos and moon rocks.
"""
+++

## markdown

text can be **bold**, *italic*, ~~strikethrough~~, and ***~~all at the same time~~***.

[link to another page](@/demo/page.md).

there should be whitespace between paragraphs[^1].

# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6

this is a normal paragraph[^2] following a header.

😭😂🥺🤣❤️✨🙏😍🥰😊

```
Long, single-line code blocks should not wrap. They should horizontally scroll if they are too long. This line should be long enough to demonstrate this.
```

> "Original content is original only for a few seconds before getting old"
> > Rule #21 of the internet

- Item 1
- Item 2
  - Item 2.1
  - Item 2.2
- Item 3
- `Item 4`

1. Perform step #1
2. Proceed to step #2
3. Conclude with step #3

- [ ] Milk
- [x] Eggs
- [x] Flour
- [ ] Coffee
- [x] Combustible lemons

[![male mallard duck with green head on water](https://upload.wikimedia.org/wikipedia/commons/thumb/2/24/Male_mallard_duck_2.jpg/800px-Male_mallard_duck_2.jpg)](https://upload.wikimedia.org/wikipedia/commons/2/24/Male_mallard_duck_2.jpg)

| Mare         | Rating            | Additional info  |
| :----------- | :---------------- | :--------------- |
| Fluttershy   | Best pone         | Shy and adorable |
| Apple Jack   | Good pone         | Honest and nice  |
| Pinkie Pie   | Fun pone          | Parties and ADHD |
| Twilight     | Main pone         | Neeerd           |
| Rainbow Dash | Yes               | Looks badass     |
| Rarity       | Fancy pone        | Generous         |
| Derpy Hooves | *M u f f i n s*   | [REDACTED]       |

```rust
let highlight = true;
```

```scss, linenos, linenostart=10, hl_lines=3-4 8-9, hide_lines=2 7
pre mark {
  // If you want your highlights to take the full width
  display: block;
  color: currentcolor;
}
pre table td:nth-of-type(1) {
  // Select a colour matching your theme
  color: #6b6b6b;
  font-style: italic;
}
```

> [!NOTE]
> useful information that users should know, even when skimming content.

> [!TIP]
> helpful advice for doing things better or more easily.

> [!IMPORTANT]
> key information users need to know to achieve their goal.

> [!WARNING]
> urgent info that needs immediate user attention to avoid problems.

> [!CAUTION]
> advises about risks or negative outcomes of certain actions.

***

## extra

### KaTeX

i can render LaTeX with the [KaTeX](https://katex.org) library, enabled by the `extra.katex` config variable.

```latex
$$\relax f(x) = \int_{-\infty}^\infty\hat{f}(\xi)\,e^{2 \pi i \xi x}\,d\xi$$
```

$$\relax f(x) = \int_{-\infty}^\infty\hat{f}(\xi)\,e^{2 \pi i \xi x}\,d\xi$$

```latex
$\relax f(x) = \int_{-\infty}^\infty\hat{f}(\xi)\,e^{2 \pi i \xi x}\,d\xi$
```

$\relax f(x) = \int_{-\infty}^\infty\hat{f}(\xi)\,e^{2 \pi i \xi x}\,d\xi$

### shortcodes

i have a few useful [shortcodes](https://www.getzola.org/documentation/content/shortcodes/) that simplify some tasks, usable on any page.

#### alerts

[github-style](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#alerts) alerts. wrap the alert text inside the shortcode to get the look.

> [!NOTE]
> since version 0.21.0, Zola supports github flavored alerts natively, without this shortcode. enable it by adding this to `config.toml`:
>
> ```toml
> [markdown]
> github_alerts = true
> ```

available alert types:

- `note`: useful information that users should know, even when skimming content.
- `tip`: helpful advice for doing things better or more easily.
- `important`: key information users need to know to achieve their goal.
- `warning`: urgent info that needs immediate user attention to avoid problems.
- `caution`: advises about risks or negative outcomes of certain actions.

```jinja2
{%/* alert(note=true) */%}
-> Alert text <-
{%/* end */%}
```

{% alert(note=true) %}
useful information that users should know, even when skimming content.
{% end %}

{% alert(tip=true) %}
helpful advice for doing things better or more easily.
{% end %}

{% alert(important=true) %}
key information users need to know to achieve their goal.
{% end %}

{% alert(warning=true) %}
urgent info that needs immediate user attention to avoid problems.
{% end %}

{% alert(caution=true) %}
advises about risks or negative outcomes of certain actions.
{% end %}

#### images and videos

by default, images and videos come with generic styling, like rounded corners and shadow. to fine-tune that, use shortcodes with different variable combinations.

available variables:

- `url`: url to an image.
- `url_min`: url to a compressed version, the original opens by clicking the image.
- `alt`: alt text, same as text inside square brackets in markdown.
- `full`: forces the image to full-width.
- `full_bleed`: forces the image to fill all available screen width. removes shadow, rounded corners and zoom on hover.
- `start`: floats the image to the start of the paragraph and scales it down.
- `end`: floats the image to the end of the paragraph and scales it down.
- `pixels`: nearest neighbor scaling, keeps pixel-art sharp.
- `transparent`: removes rounded corners and shadow, useful for images with transparency.
- `no_hover`: removes zoom on hover.
- `spoiler`: blurs the image until hovered/pressed, useful for plot rich game screenshots.
- `spoiler` with `solid`: ditto, but hides the image completely.

```jinja2
{{/* image(url="image.png", alt="This is an image", no_hover=true) */}}
```

<figure>
{{ image(url="https://i1.theportalwiki.net/img/2/23/Ashpd_blueprint.jpg", alt="portal gun blueprint", no_hover=true) }}
<figcaption>image with alt text and no zoom on hover</figcaption>
</figure>

<figure>
{{ image(url="https://upload.wikimedia.org/wikipedia/commons/b/b4/JPEG_example_JPG_RIP_100.jpg", url_min="https://upload.wikimedia.org/wikipedia/commons/3/38/JPEG_example_JPG_RIP_010.jpg", alt="the gravestone of J.P.G.", no_hover=true) }}
<figcaption>image with a compressed version, alt text, and no zoom on hover</figcaption>
</figure>

<figure>
{{ image(url="https://files.catbox.moe/lk7nee.jpg", alt="game screenshot hidden behind a spoiler", spoiler=true) }}
<figcaption>image with alt text, hidden behind a spoiler</figcaption>
</figure>

alternatively, you can append the following URL anchors. handier in some cases, e.g. these render normally in any markdown editor, unlike the Zola shortcodes.

- `#full`: forces the image to full-width.
- `#full-bleed`: forces the image to fill all available screen width. removes shadow, rounded corners and zoom on hover.
- `#start`: floats the image to the start of the paragraph and scales it down.
- `#end`: floats the image to the end of the paragraph and scales it down.
- `#pixels`: nearest neighbor scaling, keeps pixel-art sharp.
- `#transparent`: removes rounded corners and shadow, useful for images with transparency.
- `#no-hover`: removes zoom on hover.
- `#spoiler`: blurs the image until hovered/pressed, useful for plot rich game screenshots.
- `#spoiler` with `#solid`: ditto, but hides the image completely.

<br />
<figure>

[![toolbx header logo](https://containertoolbx.org/assets/toolbx.gif#full#pixels#transparent#no-hover)](https://containertoolbx.org)
<figcaption>full-width image with alt text, pixel-art rendering, no shadow or rounded corners, and no zoom on hover</figcaption>
</figure>

<br />

![white 1966 ford mustang coupe](https://upload.wikimedia.org/wikipedia/commons/thumb/1/1b/1966_Ford_Mustang_coupe_white_003.jpg/320px-1966_Ford_Mustang_coupe_white_003.jpg#start)
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim aeque doleamus animo, cum corpore dolemus, fieri tamen permagna accessio potest, si aliquod aeternum et infinitum impendere malum nobis opinemur.

\
[![lej da staz lake just before sunrise in october](https://images.unsplash.com/photo-1635410773896-da585e1fe138?q=80&w=2063&auto=format&fit=crop&ixlib=rb-4.0.3&ixid=M3wxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8fA%3D%3D#full-bleed)](https://unsplash.com/photos/a-mountain-lake-surrounded-by-trees-and-snow-CqTOTZh5vrs)

for videos it's all the same, with a few differences: the `no_hover` and `url_min` variables aren't available.

you can also set the following [attributes](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/video#attributes):

- `autoplay`: start playing the video automatically.
- `controls`: show video controls like volume, seeking and pause/resume.
- `loop`: play the video again once it ends.
- `muted`: turn off audio by default.
- `playsinline`: prevent the video from playing fullscreen by default (depends on the browser).

```jinja2
{{/* video(url="video.webm", alt="This is a video", controls=true) */}}
```

<figure>
{{ video(url="https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.webm", alt="Red flower wakes up", controls=true) }}
<figcaption>webm video example from MDN</figcaption>
</figure>

<figure>
{{ video(url="https://upload.wikimedia.org/wikipedia/commons/transcoded/0/0e/Duckling_preening_%2881313%29.webm/Duckling_preening_%2881313%29.webm.720p.vp9.webm", alt="Duckling preening", full_bleed=true, controls=true) }}
<figcaption>duckling preening</figcaption>
</figure>

#### CRT

this one doesn't simplify anything, it just adds a CRT-like effect around markdown code blocks.

```jinja2
{%/* crt() */%}
-> Markdown code block <-
{%/* end */%}
```

{% crt() %}

```
 _____________________________________________
|.'',        Public_Library_Halls         ,''.|
|.'.'',                                 ,''.'.|
|.'.'.'',                             ,''.'.'.|
|.'.'.'.'',                         ,''.'.'.'.|
|.'.'.'.'.|                         |.'.'.'.'.|
|.'.'.'.'.|===;                 ;===|.'.'.'.'.|
|.'.'.'.'.|:::|',             ,'|:::|.'.'.'.'.|
|.'.'.'.'.|---|'.|, _______ ,|.'|---|.'.'.'.'.|
|.'.'.'.'.|:::|'.|'|???????|'|.'|:::|.'.'.'.'.|
|,',',',',|---|',|'|???????|'|,'|---|,',',',',|
|.'.'.'.'.|:::|'.|'|???????|'|.'|:::|.'.'.'.'.|
|.'.'.'.'.|---|','   /%%%\   ','|---|.'.'.'.'.|
|.'.'.'.'.|===:'    /%%%%%\    ':===|.'.'.'.'.|
|.'.'.'.'.|%%%%%%%%%%%%%%%%%%%%%%%%%|.'.'.'.'.|
|.'.'.'.','       /%%%%%%%%%\       ','.'.'.'.|
|.'.'.','        /%%%%%%%%%%%\        ','.'.'.|
|.'.','         /%%%%%%%%%%%%%\         ','.'.|
|.','          /%%%%%%%%%%%%%%%\          ','.|
|;____________/%%%%%Spicer%%%%%%\____________;|
```

{% end %}

there's also a `cursor` class you can add to a span with e.g. the `█` character to simulate the terminal cursor. it doesn't work inside markdown code blocks though.

#### YouTube

embeds a YouTube video using youtube-nocookie.

available variables:

- `autoplay`: whether the video should autoplay.
- `start`: which second the video starts on.

```jinja2
{{/* youtube(id="0Da8ZhKcNKQ") */}}
```

{{ youtube(id="0Da8ZhKcNKQ") }}

#### Vimeo

embeds a Vimeo video.

available variables:

- `autoplay`: whether the video should autoplay.

```jinja2
{{/* vimeo(id="869483483") */}}
```

{{ vimeo(id="869483483") }}

#### Mastodon

embeds a Mastodon post.

available variables:

- `host`: the instance the post lives on. if not set, falls back to the one in the `[extra.comments]` section of `config.toml`.
- `user`: the poster. if not set, falls back to the one in the `[extra.comments]` section of `config.toml`.
- `id`: the post id, usually at the end of the URL.

```jinja2
{{/* mastodon(host="toot.community", user="sungsphinx", id="111789185826519979") */}}
```

{{ mastodon(host="toot.community", user="sungsphinx", id="111789185826519979") }}

### description list (`<dl>`)

```html
<dl>
<dt>Something</dt>
<dd>And its description</dd>
</dl>
```

<dl>
<dt>Name</dt>
<dd>Godzilla</dd>
<dt>Born</dt>
<dd>1952</dd>
<dt>Birthplace</dt>
<dd>Japan</dd>
<dt>Color</dt>
<dd>Green</dd>
</dl>

### form input (`<input>`)

```html
<input type="checkbox" />
<label>Checkbox</label>
```

<ul>
  <li>
    <input type="checkbox" />
    <label>&nbsp;Milk</label>
  </li>
  <li>
    <input type="checkbox" />
    <label>&nbsp;Eggs</label>
  </li>
  <li>
    <input type="checkbox" />
    <label>&nbsp;Flour</label>
  </li>
  <li>
    <input type="checkbox" checked />
    <label>&nbsp;Coffee</label>
  </li>
  <li>
    <input type="checkbox" disabled />
    <label>&nbsp;Combustible lemons</label>
  </li>
</ul>

with the `switch` class:

```html
<input class="switch" type="checkbox" />
<label>Checkbox</label>
```

<ul>
  <li>
    <input class="switch" type="checkbox" />
    <label>&nbsp;Milk</label>
  </li>
  <li>
    <input class="switch" type="checkbox" />
    <label>&nbsp;Eggs</label>
  </li>
  <li>
    <input class="switch" type="checkbox" />
    <label>&nbsp;Flour</label>
  </li>
  <li>
    <input class="switch" type="checkbox" checked />
    <label>&nbsp;Coffee</label>
  </li>
  <li>
    <input class="switch" type="checkbox" disabled />
    <label>&nbsp;Combustible lemons</label>
  </li>
</ul>

with the `switch` and `big` classes:

```html
<input class="switch big" type="checkbox" />
<label>Checkbox</label>
```

<ul>
  <li>
    <input class="switch big" type="checkbox" />
    <label>&nbsp;Milk</label>
  </li>
  <li>
    <input class="switch big" type="checkbox" />
    <label>&nbsp;Eggs</label>
  </li>
  <li>
    <input class="switch big" type="checkbox" />
    <label>&nbsp;Flour</label>
  </li>
  <li>
    <input class="switch big" type="checkbox" checked />
    <label>&nbsp;Coffee</label>
  </li>
  <li>
    <input class="switch big" type="checkbox" disabled />
    <label>&nbsp;Combustible lemons</label>
  </li>
</ul>

with the `radio` type:

```html
<input type="radio" name="test" />
<label>Radio</label>
```

<ul>
  <li>
    <input type="radio" name="test" />
    <label>&nbsp;Milk</label>
  </li>
  <li>
    <input type="radio" name="test" />
    <label>&nbsp;Eggs</label>
  </li>
  <li>
    <input type="radio" name="test" />
    <label>&nbsp;Flour</label>
  </li>
  <li>
    <input type="radio" name="test" checked />
    <label>&nbsp;Coffee</label>
  </li>
  <li>
    <input type="radio" name="test" disabled />
    <label>&nbsp;Combustible lemons</label>
  </li>
</ul>

with the `color` type:

```html
<label>Color:</label>
<input type="color" value="#000000" />
```

<label for="color">Color:</label>
<input id="color" type="color" value="#b57edc" />

<label for="color">Disabled:</label>
<input id="color" type="color" value="#b57edc" disabled />

with the `range` type:

```html
<input type="range" max="100" value="33">
```

<input type="range" max="100" value="33" id="range">
<!-- For the demo purposes only -->
<small id="range-value"></small>
<!-- End -->

### figure captions (`<figcaption>`)

```markdown
<figure>
  -> Whatever content <-
  <figcaption>Caption of content above</figcaption>
</figure>
```

<figure>

  ![the office where Stanley works, yellow floor and beige walls](https://i.ibb.co/MPDJRsT/ImMAXM3.png)
  <figcaption>the office where Stanley works, yellow floor and beige walls</figcaption>
</figure>

### accordion (`<details>`)

```markdown
<details>
  <summary>Accordion title</summary>
  -> Contents here <-
</details>
```

<details>
  <summary>reveal accordion</summary>

  get it? i know, it's an awful pun.
  ![piano accordion](https://upload.wikimedia.org/wikipedia/commons/thumb/1/1b/PianoAccordeon.jpg/916px-PianoAccordeon.jpg#transparent#no-hover)

</details>

### side comment (`<small>`)

```html
<small>Small, cute text that doesn't catch attention.</small>
```

<small>Small, cute text that doesn't catch attention.</small>

### abbreviation (`<abbr>`)

```html
<abbr title="American Standard Code for Information Interchange">ASCII</abbr>
```

the <abbr title="American Standard Code for Information Interchange">ASCII</abbr> art is awesome!

### aside (`<aside>`)

```html
<aside>

-> Contents here <-
</aside>
```

<aside>

Quill and a parchment

<img class="transparent no-hover" style="margin-block-end: 0; border-radius: 0;" alt="quill and a parchment" src="https://upload.wikimedia.org/wikipedia/commons/thumb/b/b9/%D7%A7%D7%9C%D7%A3%2C_%D7%A0%D7%95%D7%A6%D7%94_%D7%95%D7%93%D7%99%D7%95.jpg/326px-%D7%A7%D7%9C%D7%A3%2C_%D7%A0%D7%95%D7%A6%D7%94_%D7%95%D7%93%D7%99%D7%95.jpg" />
</aside>

A quill is a writing tool made from a moulted flight feather (preferably a primary wing-feather) of a large bird. Quills were used for writing with ink before the invention of the dip pen, the metal-nibbed pen, the fountain pen, and, eventually, the ballpoint pen.

As with the earlier reed pen (and later dip pen), a quill has no internal ink reservoir and therefore needs to periodically be dipped into an inkwell during writing. The hand-cut goose quill is rarely used as a calligraphy tool anymore because many papers are now derived from wood pulp and would quickly wear a quill down. However it is still the tool of choice for a few scribes who have noted that quills provide an unmatched sharp stroke as well as greater flexibility than a steel pen.

### keyboard input (`<kbd>`)

```html
<kbd>⌘ Command</kbd>.
```

to switch the keyboard layout, press <kbd>⌘ Super</kbd> + <kbd>Space</kbd>.

### mark text (`<mark>`)

```html
<mark>Marked text</mark>
```

you know what? i'm gonna say some <mark>very important</mark> stuff, so <mark>important</mark> that even **bold** is not enough.

### deleted and inserted text (`<del>` and `<ins>`)

```html
<del>Something deleted</del> <ins>Something added</ins>
```

<del>Text deleted</del> <ins>Text added</ins>

### progress indicator (`<progress>`)

```html
<progress></progress>
<progress value="33" max="100"></progress>
```

<progress></progress>
<progress value="33" max="100"></progress>

### sample output (`<samp>`)

```html
<samp>Sample Output</samp>
```

<samp>Sample Output</samp>

### inline quotation (`<q>`)

```html
<q>Someone said something</q>
```

blah blah <q>inline quote</q> hmm.

### unarticulated annotation (`<u>`)

```html
<u>Gmarrar mitsakes</u>
```

<u>Yeet</u> the <u>sus</u> drip while <u>vibing</u> with the <u>TikTok</u> <u>fam</u> on a cap-free boomerang.

### external links

```html
<a class="external" href="https://example.org">External link</a>
```

<a class="external" href="https://example.org">Link to site</a>

### spoilers

```html
<span class="spoiler">Some spoiler</span>
```

you know, <span class="spoiler">brenner is a pretty dumb name.</span> i know, crazy.

with the `solid` class:

```html
<span class="spoiler solid">Some spoiler</span>
```

you know, <span class="spoiler solid">brenner is a pretty dumb name.</span> i know, crazy.

### buttons dialog

```html.j2
<div class="buttons">
  <a href="#top">Go to Top</a>
  <a class="colored external" href="https://example.org">Example</a>
</div>
```

<div class="buttons">
  <a href="#top">Go to Top</a>
  <a class="colored external" href="https://example.org">Example</a>
</div>

with the `centered` and `big` classes:

```html.j2
<div class="buttons centered">
  <button class="big colored">Do Something…</button>
</div>
```

<div class="buttons centered">
  <button class="big colored">Do Something…</button>
</div>

[^1]: Footnote
[^2]: [Footnote (link)](https://example.org)

<!-- For the demo purposes only -->
<div id="color-picker-container">
  <small>Accent color:</small>
  <input id="color-picker-light" type="color" value="#ff7800" />
  <label for="color-picker-light">Light theme</label>
  <br />
  <input id="color-picker-dark" type="color" value="#ffa348" />
  <label for="color-picker-dark">Dark theme</label>
</div>
<!-- End -->
