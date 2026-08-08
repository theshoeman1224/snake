# Resume integration

The release in `dist/` uses relative asset URLs and can be hosted at any path on
a static website. Keep the game isolated from resume styles and JavaScript by
embedding it in an iframe.

## Export a build

Pass the resume's public game directory to the export script:

```bash
./scripts/export-to-resume.sh /path/to/resume/public/games/snake
```

The script creates a clean release from the checked-out Snake commit and copies
it into the selected directory. It does not delete existing files. Review and
commit the resulting resume repository changes manually.

## Embed the game

```html
<iframe
  class="snake-game"
  src="/games/snake/index.html"
  title="Play Snake, built with Rust and WebAssembly"
  loading="lazy"
  sandbox="allow-scripts allow-same-origin"
></iframe>
```

```css
.snake-game {
  display: block;
  width: 100%;
  min-height: 900px;
  border: 0;
}

@media (max-width: 760px) {
  .snake-game {
    min-height: 1200px;
  }
}
```

The resume can read `/games/snake/game-manifest.json` if it needs to display the
embedded commit outside the iframe. The game itself compares that commit with
GitHub and links to the source repository.
