# SEO Nodes

Search engine optimization metadata for Voce IR documents. The compiler emits `<head>` meta tags, JSON-LD structured data, and generates `sitemap.xml` and `robots.txt`. SEO metadata is attached per-page via the `metadata` field on ViewRoot.

## PageMetadata

Per-page SEO configuration. One per ViewRoot. The validator warns if `title` exceeds 60 characters or `description` exceeds 160 characters.

| Field           | Type              | Required | Description                                    |
|-----------------|-------------------|----------|------------------------------------------------|
| title           | string            | yes      | Page title (validator warns if > 60 chars)     |
| title_template  | string            | no       | Title template (e.g., "%s | My Site")          |
| description     | string            | no       | Meta description (validator warns if > 160 chars) |
| canonical_url   | string            | no       | Canonical URL for this page                    |
| robots          | RobotsDirective   | no       | Robots meta directives                         |
| open_graph      | OpenGraphData     | no       | Open Graph metadata                            |
| twitter_card    | TwitterCardData   | no       | Twitter Card metadata                          |
| alternates      | [AlternateLink]   | no       | Hreflang alternate links for i18n              |
| structured_data | [StructuredData]  | no       | JSON-LD structured data blocks                 |
| custom_meta     | [MetaTag]         | no       | Custom meta tags (escape hatch)                |

```json
{
  "title": "Voce IR Documentation",
  "title_template": "%s | Voce IR",
  "description": "Schema reference and guide for the Voce IR intermediate representation.",
  "canonical_url": "https://voce-ir.xyz/docs",
  "robots": { "index": true, "follow": true },
  "open_graph": {
    "title": "Voce IR Documentation",
    "description": "Schema reference and guide for the Voce IR intermediate representation.",
    "og_type": "Website",
    "site_name": "Voce IR"
  }
}
```

## OpenGraphData

Open Graph protocol metadata for social sharing previews.

| Field        | Type   | Required | Description                                  |
|--------------|--------|----------|----------------------------------------------|
| title        | string | no       | OG title (falls back to PageMetadata.title)  |
| description  | string | no       | OG description                               |
| image        | string | no       | Image URL or MediaNode reference             |
| image_alt    | string | no       | Alt text for the OG image                    |
| image_width  | int32  | no       | Image width in pixels                        |
| image_height | int32  | no       | Image height in pixels                       |
| og_type      | OGType | no       | Website (default), Article, Product, Profile |
| url          | string | no       | Canonical page URL                           |
| site_name    | string | no       | Site name                                    |
| locale       | string | no       | Content locale (e.g., "en_US")               |

```json
{
  "title": "Introducing Voce IR",
  "description": "An AI-native UI intermediate representation.",
  "image": "https://voce-ir.xyz/og-image.png",
  "image_alt": "Voce IR logo and tagline",
  "image_width": 1200,
  "image_height": 630,
  "og_type": "Website",
  "site_name": "Voce IR"
}
```

## StructuredData

JSON-LD structured data block for search engine rich results. The validator checks basic conformance to the declared Schema.org type.

| Field           | Type   | Required | Description                                      |
|-----------------|--------|----------|--------------------------------------------------|
| schema_type     | string | yes      | Schema.org type (e.g., "Article", "Product", "FAQ", "BreadcrumbList") |
| properties_json | string | yes      | JSON-LD properties as a JSON string              |

```json
{
  "schema_type": "Article",
  "properties_json": "{\"headline\":\"Getting Started with Voce IR\",\"author\":{\"@type\":\"Person\",\"name\":\"Voce Team\"},\"datePublished\":\"2025-01-15\"}"
}
```

## TwitterCardData

Twitter (X) Card metadata for link previews on the platform.

| Field      | Type            | Required | Description                                  |
|------------|-----------------|----------|----------------------------------------------|
| card_type  | TwitterCardType | no       | Summary, SummaryLargeImage (default), App, Player |
| title      | string          | no       | Card title                                   |
| description| string          | no       | Card description                             |
| image      | string          | no       | Image URL                                    |
| image_alt  | string          | no       | Alt text for the image                       |
| site       | string          | no       | @username of the site                        |
| creator    | string          | no       | @username of the content creator             |

## RobotsDirective

Controls search engine crawling and indexing behavior for the page.

| Field             | Type             | Required | Description                                |
|-------------------|------------------|----------|--------------------------------------------|
| index             | bool             | no       | Allow indexing (default true)              |
| follow            | bool             | no       | Follow links on page (default true)        |
| max_snippet       | int32            | no       | Max snippet length, -1 = unlimited (default) |
| max_image_preview | ImagePreviewSize | no       | None, Standard, Large (default)            |
| max_video_preview | int32            | no       | Max video preview seconds, -1 = unlimited  |
| no_archive        | bool             | no       | Prevent cached page (default false)        |
| no_translate      | bool             | no       | Prevent translation (default false)        |

## AlternateLink

Hreflang link for international SEO, connecting pages across locales.

| Field    | Type   | Required | Description                                      |
|----------|--------|----------|--------------------------------------------------|
| hreflang | string | yes      | BCP 47 language tag (e.g., "en-US", "x-default") |
| href     | string | yes      | URL of the alternate page                        |

```json
[
  { "hreflang": "en-US", "href": "https://voce-ir.xyz/en/docs" },
  { "hreflang": "fr-FR", "href": "https://voce-ir.xyz/fr/docs" },
  { "hreflang": "x-default", "href": "https://voce-ir.xyz/docs" }
]
```

## MetaTag

Custom meta tag for cases not covered by the structured fields above.

| Field       | Type   | Required | Description                                    |
|-------------|--------|----------|------------------------------------------------|
| name        | string | yes      | Meta tag name or property attribute            |
| content     | string | yes      | Meta tag content value                         |
| is_property | bool   | no       | Use property= instead of name= (default false)|
