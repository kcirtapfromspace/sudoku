# iOS App Store Publishing Guide

This guide explains how to set up automated iOS App Store publishing via GitHub Actions.

## Prerequisites

1. **Apple Developer Account** ($99/year)
2. **App Store Connect access**
3. **App registered in App Store Connect** with bundle ID `com.sudoku.app`

## Required GitHub Secrets

Add these secrets to your repository (Settings > Secrets and variables > Actions):

### App Store Connect API Key

1. Go to [App Store Connect > Users and Access > Keys](https://appstoreconnect.apple.com/access/api)
2. Generate a new API key with "App Manager" role
3. Download the `.p8` file (you can only download it once!)

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `APP_STORE_CONNECT_API_KEY_ID` | Key ID (10 characters) | Shown in App Store Connect |
| `APP_STORE_CONNECT_ISSUER_ID` | Issuer ID (UUID format) | Shown at top of Keys page |
| `APP_STORE_CONNECT_API_KEY_BASE64` | Base64-encoded `.p8` file | `cat AuthKey_XXXXXXXX.p8 \| base64` |

### Signing Certificate

1. Open Keychain Access on your Mac
2. Find your "Apple Distribution" certificate
3. Export as `.p12` file with a password

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded `.p12` file | `cat certificate.p12 \| base64` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the `.p12` file | The password you set when exporting |

### Provisioning Profile

1. Go to [Apple Developer > Certificates, IDs & Profiles](https://developer.apple.com/account/resources/profiles/list)
2. Create an "App Store" distribution profile for `com.sudoku.app`
3. Download the `.mobileprovision` file

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `APPLE_PROVISIONING_PROFILE_BASE64` | Base64-encoded profile | `cat profile.mobileprovision \| base64` |

### Team ID

| Secret Name | Description | How to Get |
|-------------|-------------|------------|
| `APPLE_TEAM_ID` | Your Apple Developer Team ID | [Developer Account > Membership](https://developer.apple.com/account) |

## Workflow Triggers

### Automatic Deployments

- **Pull Requests**: Build and test only (no signing required)
- **Push to main**: Automatically deploys to TestFlight

### Manual Deployments

Use the "Run workflow" button in GitHub Actions to trigger a manual deployment.

## Local Development

### Install fastlane

```bash
cd ios/Sudoku
bundle install
```

### Build Debug

```bash
bundle exec fastlane build_debug
```

### Deploy to TestFlight (with automatic signing)

```bash
bundle exec fastlane beta
```

### Check Version

```bash
bundle exec fastlane version_info
```

## App Store Submission

To submit to the App Store for review:

```bash
bundle exec fastlane release submit:true
```

Or without auto-submit (just upload):

```bash
bundle exec fastlane release
```

## Troubleshooting

### "No signing identity found"

Make sure you've:
1. Exported the correct certificate (Apple Distribution, not Development)
2. The certificate hasn't expired
3. The base64 encoding is correct

### "Profile doesn't match bundle identifier"

Ensure your provisioning profile is for:
- Bundle ID: `com.sudoku.app`
- Distribution type: App Store

### "API key not authorized"

The API key needs "App Manager" or "Admin" role in App Store Connect.

## Security Notes

- Never commit secrets to the repository
- Use GitHub's encrypted secrets only
- Rotate certificates and API keys periodically
- The `.p8` API key file can only be downloaded once from Apple

## Architecture

```
GitHub Actions
     │
     ├── PR: Build & Test (no signing)
     │
     └── Push to main: Build → Sign → Upload to TestFlight
                               │
                               ├── Certificate (from secrets)
                               ├── Provisioning Profile (from secrets)
                               └── API Key (from secrets)
```
