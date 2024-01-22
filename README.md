![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/perpetualcacophony/slimebot/docker-publish.yml) ![Static Badge](https://img.shields.io/badge/rust-1.75.0-f74c00?logo=rust&link=https%3A%2F%2Fblog.rust-lang.org%2F2023%2F12%2F28%2FRust-1.75.0.html)

# 🌸 slimebot 🌸
slimebot is a small self-hosted discord bot made for a private server with friends.

## 🐞 coming from the server?
hey, thanks for checking out the code!! if you have a feature to request or a bug to report, you can always dm it to me directly, but i would really really appreciate if you put it in the [issues](https://github.com/perpetualcacophony/slimebot/issues) page.

### want to contribute?
developing this bot *is* fun, but does take a good amount of time and effort, so contributing would be super helpful!! the bot itself is entirely written in rust, which i can absolutely help you learn if you're interested, but in the future there might be additional features that involve web development.

### what's with slimebot-dev?
slimebot-dev also runs on this codebase! slimebot proper runs on an actual webserver that lets it stay up all the time, while slimebot-dev just runs off my computer. additionally, slimebot proper runs on the stable code in the [`prod`](https://github.com/perpetualcacophony/slimebot/tree/prod) branch, while slimebot-dev runs on whatever unstable branch i'm currently writing and testing. slimebot-dev exists so i can develop the bot while keeping your experience using slimebot relatively seamless!

## 🐞 coming from somewhere else?
hi!! this bot (and the server it's built for) is riddled with in-jokes and dumb features. it *is* a bot that *does* work—the [`prod`](https://github.com/perpetualcacophony/slimebot/tree/prod) branch is, at least—and you could probably deploy it to your own hardware, but you're probably just better off taking what you like from the codebase. unless you *want* a bot that posts [this image of joe biden](https://files.catbox.moe/v7itt0.webp) every time someone says "L", i guess?

### can i use this bot in my own server?
yes and no. you're completely free to compile and run the code yourself, or use the docker image at [`ghcr.io/perpetualcacophony/slimebot:prod`](https://ghcr.io/perpetualcacophony/slimebot) (check out the example [`compose.yaml`](example-compose.yaml)!) however, you'll need to use your own bot user—the bot application i operate is private server-only, which is why you won't find any invite link for slimebot.
