import * as core from '@actions/core'
import * as github from '@actions/github'
import {
  EmbedBuilder,
  WebhookClient,
  WebhookMessageCreateOptions,
  roleMention
} from 'discord.js'

/**
 * The main function for the action.
 * @returns {Promise<void>} Resolves when the action is complete.
 */
export async function run(): Promise<void> {
  try {
    const webhookUrl = core.getInput('webhook-url', { required: true })
    const webhook = new WebhookClient({ url: webhookUrl })

    // the following parts have been copied from the checkout action
    // - https://github.com/actions/checkout/blob/1d96c772d19495a3b5c517cd2bc0cb401ea0529f/src/input-helper.ts#L20-L72
    const qualifiedRepository =
      core.getInput('repository') ||
      `${github.context.repo.owner}/${github.context.repo.repo}`
    core.debug(`qualified repository = '${qualifiedRepository}'`)

    const isWorkflowRepo =
      qualifiedRepository.toUpperCase() ===
      `${github.context.repo.owner}/${github.context.repo.repo}`.toUpperCase()

    const splitRepository = qualifiedRepository.split('/')
    if (
      splitRepository.length !== 2 ||
      !splitRepository[0] ||
      !splitRepository[1]
    ) {
      throw new Error(
        `Invalid repository '${qualifiedRepository}'. Expected format {owner}/{repo}.`
      )
    }
    const repositoryOwner = splitRepository[0]
    const repositoryName = splitRepository[1]
    core.debug(`repository owner = '${repositoryOwner}'`)
    core.debug(`repository name = '${repositoryName}'`)

    const projectName =
      core.getInput('project-name', { required: false }) || isWorkflowRepo
        ? repositoryName
        : undefined
    if (!projectName) {
      throw new Error(
        'project-name is required if the provided repository is not repository running the workflow!'
      )
    }

    let version = core.getInput('version', { required: false })
    if (!version && isWorkflowRepo) {
      const ref = github.context.ref?.replace('refs/tags/', '')
      if (ref) {
        version = /v\d/.test(ref) ? ref.substring(1) : ref
      }
    }

    const modrinthId = core.getInput('modrinth-project-id', { required: false })
    const curseforgeId = core.getInput('curseforge-project-id', {
      required: false
    })

    const thumbnailUrl = core.getInput('thumbnail-url', { required: false })

    const shouldPingRoleValue = core.getInput('ping-notification-role', {
      required: false
    })
    let shouldPingRole = false
    if (shouldPingRoleValue === '') {
      shouldPingRole =
        !/[+-_]alpha/i.test(version) &&
        !/[_+-]beta/i.test(version) &&
        !/[_+-]rc.*/i.test(version) &&
        !/[_+-]pre-?(release)?/i.test(version) &&
        !/[_+-]snapshot.*/i.test(version) &&
        !/[_+-]dev.*/i.test(version)
    } else {
      shouldPingRole = shouldPingRoleValue !== 'false'
    }

    let notificationRole = core.getInput('notification-role-id', {
      required: false
    })
    if (notificationRole === '') {
      notificationRole = '918884941461352469'
    }

    const description: string[] = [`# ${projectName} ${version}`, '']

    if (modrinthId || curseforgeId) {
      description.push('## Downloads:')
      let downloads = ''
      if (curseforgeId) {
        if (downloads.length > 0) {
          downloads += ' | '
        }
        downloads += `<:curseforge:1231714919561429023> [Curseforge](https://www.curseforge.com/projects/${curseforgeId})`
      }
      if (modrinthId) {
        if (downloads.length > 0) {
          downloads += ' | '
        }
        downloads += `<:modrinth:1231714923503943710> [Modrinth](https://modrinth.com/mod/${modrinthId})`
      }
      description.push(downloads)
    }

    description.push('')
    description.push(
      `<:github:1231714921331425310> [Source Code](https://github.com/${qualifiedRepository})`
    )

    let embed = new EmbedBuilder()
      .setColor('#8dcf88')
      // .setAuthor({
      //   name: 'Up-Mods',
      //   // profile icon of github org
      //   iconURL: 'https://avatars.githubusercontent.com/u/141473891?s=64',
      //   url: 'https://github.com/Up-Mods'
      // })
      .setTimestamp(new Date())
      .setDescription(description.join('\n'))

    if (thumbnailUrl) {
      embed = embed.setThumbnail(thumbnailUrl)
    }

    const avatar = {
      username: 'Mod Updates',
      avatarURL: 'https://avatars.githubusercontent.com/u/141473891?s=256'
    }
    // TODO this cannot currently be crossposted; for that we would need a real bot, not just a webhook :(
    const payload: WebhookMessageCreateOptions = {
      ...avatar,
      embeds: [embed]
    }

    const result = await webhook.send(payload)
    core.debug(`Sent message with id ${result.id}`)
    core.setOutput('message', result)

    // ping the announcements role
    if (shouldPingRole && notificationRole) {
      core.debug(`Pinging role ${notificationRole}`)
      await webhook.send({
        ...avatar,
        content: roleMention(notificationRole),
        allowedMentions: {
          roles: [notificationRole]
        }
      })
    }
  } catch (error) {
    // Fail the workflow run if an error occurs
    if (error instanceof Error) core.setFailed(error.message)
  }
}
