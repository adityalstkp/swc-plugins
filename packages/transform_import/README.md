# transform_import

### Installation
```sh
pnpm add -D @adityals/swc-plugin-transform-import
```

### Usage
```javascript
{
  jsc: {
    experimental: {
      plugins: [
        require.resolve("@adityals/swc-plugin-transform-import"),
        {
          antd: {
            transform: "antd/lib/[[member]]",
            stylePath: "antd/lib/[[member]]/style",
            transformCase: "kebab_case",
          },
          lodash: {
            transform: "lodash/[[member]]",
            transformCase: "",
          },
        },
      ];
    }
  }
}
```
