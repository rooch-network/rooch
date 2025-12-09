# Parameter Compatibility Test Examples

## New Parameter (Preferred)
```bash
rooch server start -n local --traffic-replenish-interval-s 0.1 --traffic-burst-size 200
```

## Old Parameter (Still Supported - Shows Warning)
```bash
rooch server start -n local --traffic-per-second 0.1 --traffic-burst-size 200
```

## Both Parameters (New Takes Precedence)
```bash
rooch server start -n local --traffic-replenish-interval-s 0.1 --traffic-per-second 0.2 --traffic-burst-size 200
# Result: Uses 0.1 (from new parameter), shows no warning
```

## Expected Behavior
- New parameter `--traffic-replenish-interval-s`: Clear name, no warning
- Old parameter `--traffic-per-second`: Deprecated, shows warning
- Both specified: New parameter takes precedence, no warning
- Neither specified: Uses default values

## Rate Limiting Examples
- `0.1` = 10 requests per second (1 request every 0.1s)
- `1.0` = 1 request per second (1 request every 1s)
- `0.01` = 100 requests per second (1 request every 0.01s)