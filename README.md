# Twitch Launcher

CLI Twitch browser/launcher to be used with streamlink and Chatterino.

![example run](https://user-images.githubusercontent.com/18398887/183257473-80f9c9ec-2b72-4814-8f69-93cfb1772537.gif)

Currently only opens the stream with [streamlink](https://github.com/streamlink/streamlink) and the chat with [Chatterino](https://github.com/Chatterino/chatterino2) (or [Chatterino7](https://github.com/SevenTV/chatterino7)).

## Set Up

You will need:
- Twitch username
- Twitch User ID (use [this](https://www.streamweasels.com/tools/convert-twitch-username-to-user-id/) to find it)
- Twitch app Client ID
- Twitch app Client Secret
- Twitch app redirect URL port

You will be prompted for these when running the app for the first time.

You can generate the Client ID, Secret and redirect URL by [registering an app with Twitch](https://dev.twitch.tv/docs/authentication/register-app). Set the redirect URL to `http://localhost:<some_port>/` **with** a trailing slash.

Paste each of them as prompted, which will open a page on your browser to get the required tokens from Twitch. You might need to confirm the action.

Once this is done, you will be logged in and ready to use the app.
