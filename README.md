# Discord Release Action

Send notifications about new releases to a discord channel

## Usage
This is a common usage example.
Only `webhook-url` is required, everything else is optional, tho usually you want to specify at least the Curseforge or Modrinth project IDs.
```yml
- uses: Up-Mods/action-discord-release@main
  with:
    webhook-url: ${{ secrets.ANNOUNCEMENT_WEBHOOK_URL }}
    curseforge-project-id: 428151
    modrinth-project-id: Dw7M6XKW
    thumbnail-url: https://mod-assets.upcraft.dev/promo/icarus/icon_128x.png
```

### Inputs
This is a list of ALL possible inputs, their default values, and whether they are required.

> [!NOTE]
>
> A note on emoji IDs for discord:
>
> To use built-in emoji, use `:emoji:` or their unicode representation, for example `:fire:`​
> For custom emoji, use the format `<:emoji:ID>`, for example `<:github:1231714921331425310>` (You can find an emoji's ID by right clicking it in the emoji picker while having developer mod enabled)

```yml
- uses: Up-Mods/action-discord-release@main
  with:
    # Display name of the project [OPTIONAL]
    # if not specified, attempts to parse it from the source code URL.
    project-name:
    
    # Version to be published [OPTIONAL]
    # defaults to the current GitHub ref name.
    version:
    
    # URL to find the project source code at [OPTIONAL]
    # defaults to the current GitHub repository URL.
    sourcecode-url:
    
    # URL to post the webhook to [REQUIRED]
    # Note that this should NOT end in '/github'.
    webhook-url:
    
    # The display name for the webhook [OPTIONAL]
    # defaults to 'Mod Updates'
    webhook-username:
    
    # The URL to use for the webhook's avatar image [OPTIONAL]
    # defaults to https://avatars.githubusercontent.com/u/141473891?s=128
    # A size of at least 128x128 is recommended.
    webhook-avatar-url:
    
    # URL to find the project thumbnail at [OPTIONAL]
    # A size of at least 128x128 is recommended.
    thumbnail-url:
    
    # The role ID to ping for notifications [OPTIONAL]
    # defaults to '918884941461352469' (you probably want to change this)
    # If specified, this MUST be a role ID, not a user ID!
    # Special notation must be used for '@everyone' or '@here'!
    notification-role-id:
    
    # Whether to ping the notification role [OPTIONAL]
    # If not specified, will make a best-effort attempt to parse the 'version' input to determine whether it is a prerelease.
    # by default, will only ping if the version is not a prerelease.
    ping-notification-role:
    
    # The ID of the project on Curseforge [OPTIONAL]
    curseforge-project-id:
    
    # The ID of the project on Modrinth [OPTIONAL]
    modrinth-project-id:
    
    # The emoji to use for the source code link [OPTIONAL]
    # defaults to <:github:1231714921331425310>
    sourcecode-emoji:
    
    # The emoji to use for the source code link [OPTIONAL]
    # defaults to <:curseforge:1231714919561429023>
    curseforge-emoji:
    
    # The emoji to use for the source code link [OPTIONAL]
    # defaults to <:modrinth:1231714923503943710>
    modrinth-emoji:
```

<br><br>

### Outputs

| Name              | Description                                               | Example                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ----------------- | --------------------------------------------------------- |-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `response_status` | The status code received from the webhook                 | 204                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `message`         | The message that was sent to the webhook (in JSON format) | ```json {"avatar_url":"https://avatars.githubusercontent.com/u/141473891?s=256","embeds":[{"color":9293704,"description":"# Icarus 1.0.0\n\n## Downloads:\n:fire: [Curseforge](https://mods.cf/428151)\n\n:information: [Source Code](https://github.com/CammiesCorner/Icarus)","type":"rich","thumbnail":{"url":"https://mod-assets.upcraft.dev/promo/icarus/icon_128x.png"},"timestamp":"2026-01-01T16:32:48.484784+00:00"}],"username":"Mod Updates"}``` |

