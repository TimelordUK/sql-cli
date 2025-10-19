# Quick Start Guide

Get this running in 5 minutes on your work PC.

## 1. Create Keytab (One Time Setup)

```bash
cd docker/flask-kerberos
mkdir secrets

# Create keytab
ktutil
addent -password -p YOUR_USERNAME@YOUR.REALM.COM -k 1 -e aes256-cts
[Enter your Kerberos password]
wkt secrets/app.keytab
quit

# Secure it
chmod 600 secrets/app.keytab
```

## 2. Configure

Edit `docker-compose.yml` - change this line:

```yaml
- KRB5_PRINCIPAL=YOUR_USERNAME@YOUR.REALM.COM
```

Replace with your actual Kerberos principal.

## 3. Test Keytab

```bash
# Make sure it works
kinit -kt secrets/app.keytab YOUR_USERNAME@YOUR.REALM.COM
klist
# You should see valid tickets
```

## 4. Run

```bash
# Start the container
docker-compose up --build

# Or run in background
docker-compose up -d --build
```

## 5. Verify

```bash
# Check health endpoint
curl http://localhost:5000/health

# Should show:
# {
#   "status": "healthy",
#   "kerberos": {
#     "has_valid_ticket": true,
#     ...
#   }
# }
```

## 6. View Logs

```bash
# Renewal logs (shows ticket refreshes every 7 hours)
tail -f logs/krb5_renew.log

# Container logs
docker-compose logs -f flask-app
```

## Troubleshooting

### "Keytab file not found"
```bash
# Check keytab exists
ls -la secrets/app.keytab

# Should show:
# -rw------- 1 youruser yourgroup ... app.keytab
```

### "Failed to renew ticket"
```bash
# Test keytab manually
kinit -kt secrets/app.keytab YOUR_USERNAME@YOUR.REALM.COM

# If this fails, recreate the keytab (Step 1)
```

### "Container keeps restarting"
```bash
# Check logs
docker-compose logs flask-app

# Common fixes:
# 1. Wrong principal name in docker-compose.yml
# 2. Keytab doesn't match principal
# 3. Need to mount /etc/krb5.conf if custom config
```

## Configuration

Everything is in `docker-compose.yml`:

- **Principal**: Change `KRB5_PRINCIPAL` to match your username/realm
- **Renewal time**: Change `KRB5_RENEWAL_INTERVAL` (default: 7 hours = 25200 seconds)
- **Port**: Change `5000:5000` to use different port

## Using with Your Flask App

1. Replace `app.py` with your actual Flask application
2. Update `requirements.txt` with your dependencies
3. The Kerberos renewal runs in the background automatically
4. Your app will always have valid tickets without restarting

That's it! Your Flask app now auto-renews Kerberos tickets every 7 hours.
