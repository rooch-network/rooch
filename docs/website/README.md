# Rooch Website

The website is built with [Nextra framework](https://nextra.site/).
The site is deployed on and served by [Vercel](https://vercel.com/).

## Dependecise and Development

It is recommended to use [pnpm](https://pnpm.io/).

To install dependencise:
```
pnpm install
```

To preview the website locally:
```
pnpm dev
```

## File Structure

All editable pages are under `/pages/`.  

```
├── blog                      // all the Blog Posts go into here
│   ├── post_name.en-US.mdx   // this is a post in English
│   ├── post_name.zh-CN.mdx   // this is a post in Chinese
├── blog-template             // here you can find a blog post template with necessary metadat and post header
│   └── post-template.en-US.mdx
├── docs                      // all the documentations are in here; they should be nested with folder structures 
│   ├── _meta.en-US.json      // each of the folders in the docs should contain a _meta.json file with locale in the file name
...
└── index.en-US.mdx           
```

## Contribution

To contribute articles or documents, submit a PR directly to the `main` branch
Currently the website has both English and Chinese versions. 
To contribute docs in a specific language please follow the naming convention `doc.locale.mdx`
