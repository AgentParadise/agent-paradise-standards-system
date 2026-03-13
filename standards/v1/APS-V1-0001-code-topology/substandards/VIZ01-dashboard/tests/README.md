# VIZ01-dashboard Tests

## Unit Tests

Each visualization module includes unit tests that verify:
- HTML output starts with `<!DOCTYPE html>`
- Contains expected title
- Embeds provided data correctly

Run tests with:

```bash
cargo test -p code-topology-viz
```

## Integration Tests

Integration tests should verify:
1. Complete HTML is valid and parseable
2. Embedded JSON is correctly escaped
3. All required CSS/JS is present
4. Visualizations render correctly in browser (manual testing)

## Browser Testing

For visual verification:

```bash
# Generate sample visualizations
aps run topology analyze .
aps run topology viz --type all

# Open each and verify:
# - Page loads without console errors
# - Interactions work (pan, zoom, hover, click)
# - Data is displayed correctly
# - Responsive layout works
```
