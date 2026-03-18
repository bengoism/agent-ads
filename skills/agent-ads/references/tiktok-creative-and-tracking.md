# TikTok Creative Assets & Tracking

## Creative Assets

### Search videos

```bash
# List all video assets
agent-ads tiktok creatives videos --advertiser-id 1234567890

# With pagination
agent-ads tiktok creatives videos \
  --advertiser-id 1234567890 \
  --page-size 20 --all

# With filtering
agent-ads tiktok creatives videos \
  --advertiser-id 1234567890 \
  --filter '{"material_ids":["video123"]}'
```

### Get image info

```bash
# Get info for specific images
agent-ads tiktok creatives images \
  --advertiser-id 1234567890 \
  --image-id img123,img456
```

## Pixels

### List pixels

```bash
agent-ads tiktok pixels list --advertiser-id 1234567890

# Auto-paginate
agent-ads tiktok pixels list \
  --advertiser-id 1234567890 \
  --all
```

## Audiences

### List custom audiences

```bash
agent-ads tiktok audiences list --advertiser-id 1234567890

# With pagination
agent-ads tiktok audiences list \
  --advertiser-id 1234567890 \
  --page-size 50 --all
```
