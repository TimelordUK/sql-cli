# Flask + Kerberos Docker Example

This is a production-ready pattern for running Flask applications with Kerberos authentication in Docker. It handles automatic ticket renewal without needing to restart your application.

## Features

- Automatic Kerberos ticket renewal every 7 hours (configurable)
- Secure keytab-based authentication (no passwords in plain text)
- Health endpoint showing Kerberos ticket status
- Logs for debugging Kerberos issues
- Docker Compose setup for easy deployment

## Directory Structure

```
flask-kerberos/
├── app.py                  # Flask application
├── requirements.txt        # Python dependencies
├── Dockerfile             # Container definition
├── docker-compose.yml     # Docker Compose configuration
├── entrypoint.sh         # Container startup script
├── kerberos_renew.sh     # Ticket renewal script
├── secrets/              # Keytab files (create this)
│   └── app.keytab       # Your keytab file (NOT in git)
└── logs/                 # Renewal logs (created automatically)
    └── krb5_renew.log
```

## Setup Instructions

### Step 1: Create a Keytab File

On your work PC (where you normally run `kinit`), create a keytab:

```bash
# Create secrets directory
mkdir -p secrets

# Create keytab using ktutil
ktutil
> addent -password -p your_username@YOUR.REALM.COM -k 1 -e aes256-cts
Password for your_username@YOUR.REALM.COM: [enter your password]
> wkt secrets/app.keytab
> quit

# Secure the keytab file
chmod 600 secrets/app.keytab
```

**Alternative method** (if you have existing credentials):

```bash
# If you already have a ticket, you can extract it
kinit your_username@YOUR.REALM.COM
ktutil
> rkt /tmp/krb5cc_XXXX  # Your current ticket cache
> wkt secrets/app.keytab
> quit
```

**IMPORTANT**: Never commit the keytab file to git! It's like a password.

### Step 2: Configure Your Environment

Edit `docker-compose.yml` and update:

```yaml
environment:
  - KRB5_PRINCIPAL=your_username@YOUR.REALM.COM  # Your Kerberos principal
  - KRB5_RENEWAL_INTERVAL=25200                  # 7 hours (adjust as needed)
```

If you have a custom `/etc/krb5.conf`, ensure it's mounted correctly. If not, comment out that volume mount.

### Step 3: Test the Keytab

Before running Docker, verify your keytab works:

```bash
# Test authentication
kinit -kt secrets/app.keytab your_username@YOUR.REALM.COM

# Check ticket
klist

# Should show your principal and ticket expiration time
```

### Step 4: Run the Application

```bash
# Build and start the container
docker-compose up --build

# Or run in background
docker-compose up -d --build
```

## Testing

### Check Health Endpoint

```bash
# From your host machine
curl http://localhost:5000/health

# You should see JSON with Kerberos status
{
  "status": "healthy",
  "timestamp": "2025-10-19T12:34:56",
  "kerberos": {
    "has_valid_ticket": true,
    "ticket_cache": "/tmp/krb5cc_flask",
    "details": "Ticket cache: FILE:/tmp/krb5cc_flask\n..."
  }
}
```

### View Renewal Logs

```bash
# Logs are mounted to ./logs/ directory
tail -f logs/krb5_renew.log

# Or view Docker logs
docker-compose logs -f flask-app
```

### Check Ticket Inside Container

```bash
# Get a shell in the running container
docker exec -it flask-kerberos-demo bash

# Check current tickets
klist

# You should see valid tickets for your principal
```

## Configuration Options

All configuration is in `docker-compose.yml`:

| Environment Variable | Description | Default |
|---------------------|-------------|---------|
| `KRB5_PRINCIPAL` | Your Kerberos principal | `your_username@YOUR.REALM.COM` |
| `KRB5_KEYTAB` | Path to keytab inside container | `/etc/krb5.keytab` |
| `KRB5_RENEWAL_INTERVAL` | Seconds between renewals | `25200` (7 hours) |
| `KRB5_LOGFILE` | Renewal log location | `/var/log/krb5_renew.log` |
| `KRB5CCNAME` | Ticket cache location | `/tmp/krb5cc_flask` |

### Adjusting Renewal Interval

Your ticket lifetime depends on your Kerberos server configuration. To check:

```bash
# After kinit, check ticket lifetime
klist

# Look for the "expires" time
# Default ticket lifetime: Valid starting     Expires            Service principal
# 10/19/25 08:00:00  10/19/25 18:00:00  krbtgt/YOUR.REALM.COM@YOUR.REALM.COM
```

Set `KRB5_RENEWAL_INTERVAL` to slightly less than your ticket lifetime. For example:
- 10-hour tickets → use 7-9 hours (25200-32400 seconds)
- 24-hour tickets → use 20-22 hours (72000-79200 seconds)

## Troubleshooting

### No Valid Ticket

**Check logs:**
```bash
# View renewal logs
cat logs/krb5_renew.log

# View container logs
docker-compose logs flask-app
```

**Common issues:**
1. Keytab file not found or not mounted correctly
2. Wrong principal name in `KRB5_PRINCIPAL`
3. Incorrect Kerberos realm configuration

### Permission Denied on Keytab

```bash
# Ensure correct permissions
chmod 600 secrets/app.keytab

# Check ownership (should be your user)
ls -la secrets/app.keytab
```

### Kerberos Config Not Found

If you need a custom `krb5.conf`:

```bash
# Create a custom config
cat > krb5.conf <<EOF
[libdefaults]
    default_realm = YOUR.REALM.COM
    dns_lookup_realm = false
    dns_lookup_kdc = false

[realms]
    YOUR.REALM.COM = {
        kdc = kdc.your-domain.com
        admin_server = kdc.your-domain.com
    }

[domain_realm]
    .your-domain.com = YOUR.REALM.COM
    your-domain.com = YOUR.REALM.COM
EOF

# Mount it in docker-compose.yml
volumes:
  - ./krb5.conf:/etc/krb5.conf:ro
```

### Container Keeps Restarting

```bash
# Check what's failing
docker-compose logs flask-app

# Common issues:
# 1. Flask app error (check app.py syntax)
# 2. Keytab authentication failing (check principal/keytab)
# 3. Port already in use (change port in docker-compose.yml)
```

### Testing Ticket Renewal

To test renewal without waiting 7 hours:

```bash
# Edit docker-compose.yml temporarily
environment:
  - KRB5_RENEWAL_INTERVAL=300  # 5 minutes for testing

# Restart
docker-compose down
docker-compose up

# Watch logs
tail -f logs/krb5_renew.log
```

## Security Best Practices

1. **Never commit secrets:**
   ```bash
   # Add to .gitignore
   echo "secrets/" >> .gitignore
   echo "*.keytab" >> .gitignore
   ```

2. **Restrict keytab permissions:**
   ```bash
   chmod 600 secrets/app.keytab
   ```

3. **Use Docker secrets in production:**
   ```yaml
   # For production, use Docker secrets instead of volume mounts
   secrets:
     krb5_keytab:
       file: ./secrets/app.keytab

   services:
     flask-app:
       secrets:
         - krb5_keytab
   ```

4. **Rotate keytabs periodically** - create new ones every few months

## Using This Pattern for Your App

To adapt this for your real Flask application:

1. **Replace `app.py`** with your actual Flask app
2. **Update `requirements.txt`** with your dependencies
3. **Modify `entrypoint.sh`** if you need additional startup steps
4. **Keep the Kerberos renewal mechanism** - it works independently

Example for your app:

```python
# Your actual Flask app
from flask import Flask
import your_kerberos_library  # e.g., requests-kerberos

app = Flask(__name__)

@app.route('/api/data')
def get_data():
    # Your Kerberos-authenticated code here
    # The ticket is automatically renewed by the background process
    response = requests.get('https://your-api.com', auth=HTTPKerberosAuth())
    return response.json()

# Keep the health endpoint for monitoring
@app.route('/health')
def health():
    # ... (use the health check from app.py)
```

## Maintenance

### Updating Your Password

When your password changes, recreate the keytab:

```bash
# Remove old keytab
rm secrets/app.keytab

# Create new keytab (follow Step 1)
ktutil
> addent -password -p your_username@YOUR.REALM.COM -k 1 -e aes256-cts
> wkt secrets/app.keytab
> quit

# Restart container
docker-compose restart
```

### Viewing Current Status

```bash
# Quick status check
curl http://localhost:5000/health | jq .

# Detailed logs
docker-compose logs --tail=50 flask-app

# Renewal log
tail -20 logs/krb5_renew.log
```

## Production Deployment

For production, consider:

1. **Use Docker secrets** instead of volume mounts
2. **Set up monitoring** on the `/health` endpoint
3. **Configure log rotation** for `krb5_renew.log`
4. **Use orchestration** (Kubernetes, Docker Swarm) for high availability
5. **Set resource limits** in docker-compose.yml

Example resource limits:

```yaml
services:
  flask-app:
    # ... other config ...
    deploy:
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
```

## License

This is example code - use and modify as needed for your project.
