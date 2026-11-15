# Independent screenshot review B

**Overall: 7.8/10.** A consistent, calm desktop interface with clear selection, useful empty states, and good sign-in recovery. It falls short of excellent chiefly through narrow-window layout, small secondary type, weak utility-control affordances, and mismatched wide-window composition.

Reviewed every one of the 36 screens in both themes, 72 original PNGs total. Read only the supplied screen descriptions and image inputs. Scores combine both themes. Screenshot review cannot establish keyboard behavior, screen-reader support, animation quality, or actual interaction target bounds.

| Dimension | Score |
| --- | ---: |
| hierarchy | 8 |
| spacing | 7.5 |
| typography | 7.5 |
| consistency | 8.5 |
| contrast accessibility | 8 |
| feedback | 8 |
| states | 8.5 |
| navigation | 8 |

## Contrast evidence

Used Python 3 isolated mode and Pillow on the original PNG pixels, with the WCAG relative-sRGB luminance formula. Selected flat foreground colors supported by repeated glyph pixels, rather than treating antialiased edge pixels as the intended foreground.

- Chat timestamp: #62656d on #ffffff, **5.83:1**.
- Selected chat timestamp: #62656d on #e4e4e8, **4.60:1**; dark #9c9ea8 on #34353c, **4.57:1**.
- Dark settings helper: #9c9ea8 on #25262b, **5.66:1**.
- Chat links: #1f63c6 on white, **5.74:1**; dark #7cb4f5 on #17181b, **8.20:1**.
- Primary sign-in text: white on #a9530f, **5.34:1**; dark #1c1208 on #e88c3e, **7.24:1**.
- Error text: #c42b2b on white, **5.63:1**; dark #f27474 on #25262b, **5.41:1**.
- Sampled light avatar initials: white on colored fills, **4.85:1 to 5.64:1**.
- Input/checkbox boundaries: #8a8d95 on white, **3.32:1**; dark #6e7179 on #25262b, **3.09:1**.
- **Settings scrollbar fails the 3:1 non-text threshold:** #eeeef0 / #f6f6f7, **1.07:1**, and #2c2d33 / #17181b, **1.29:1**. Samples at x=890, y=300 and y=500.
- Secondary button borders are weak: **1.33:1** light and **1.17:1** dark. Their readable labels may independently identify them as controls; this is an affordance finding, not a blanket WCAG failure.

No text failure was established by these samples, so the rubric's text-failure score cap was not invoked. These are representative measurements, not certification of every glyph or WCAG 2.2 criterion.

## Prioritized fixes

1. **Fixed navigation widths squeeze the task pane and truncate identity information.** (medium; both). Below 900 points, reduce the list to 200 points and offer a collapsible list with a labeled Back control. Preserve full person names in the selected header; use 16-point task-pane padding.

2. **The scrollbar thumb and track do not meet the 3:1 non-text contrast threshold.** (medium; both). Use an 8-point thumb, #737780 against #eeeef0 in light mode and #9c9ea8 against #17181b in dark mode. Keep the track visible and provide 8 points of separation from labels.

3. **Helper and metadata text is too small for comfortable desktop reading, even though sampled colors pass contrast.** (medium; both). Use 12-point metadata and 13-point helper text with 18-point line spacing. Reduce persistent composer help to Enter to send and Shift+Enter for a new line; move formatting syntax behind a help control.

4. **Reading and composing use different horizontal constraints on wide windows.** (medium; both). Place messages and composer in the same 960-point maximum-width column with matching 16-point insets. Align the send action with the reading column's right edge.

5. **The long-draft viewport cuts through the first visible line and packs many lines tightly against its edges.** (medium; both). Use 14-point draft text on a 20-point line height, 12-point internal padding, and an 8-point visible scrollbar. Keep the caret line fully visible and use a subtle top scroll shadow outside the text ink.

6. **Attachment thumbnails crop away enough content to make pending-image verification difficult.** (medium; both). Contain each image in a 144 by 96-point neutral tile rather than cropping. Add a 12-point filename or dimensions below it and a clearly accessible preview action; keep remove buttons at least 24 by 24 points.

7. **Search results visually outweigh the selected recipient, making the pending chat target less obvious.** (medium; both). Add a persistent 12-point To label and a separate selected-recipient row with 36-point chips. Label the search field Find another person. Put the existing-chat explanation below the button with an 8-point gap at 12 points.

8. **The calls landing page devotes most of its task area to explanation instead of immediate actions.** (medium; both). Keep Join a meeting first, follow with one 13-point sentence explaining browser handoff, and put capability details in the existing disclosure. Use 40-point contact rows with full names and reveal a labeled Call action for the selected person.

9. **The call panel repeats the meeting identity across two stacked headers.** (low; both). Combine status and conversation identity into one header, with a 15-point meeting title and 12-point live status. Place Join on phone, Restore, and Close bar in a 32-point action row below, targeting an 88-point total header.

10. **Secondary actions have faint outlines and often look like ordinary labels.** (low; both). Give outlined secondary actions a 1-point #6e7179 border in dark mode and #8a8d95 in light mode, with at least 32-point height and 12-point horizontal padding. Preserve stronger fill for the primary action.

11. **The preview dialog is much taller than its panoramic content needs.** (low; both). Adapt the dialog's initial height to the fitted image plus a 72-point toolbar and padding, with a 320-point minimum. Keep an explicit Maximize action for inspection.

12. **The permission title communicates an extra step rather than the capability being granted.** (low; both). Use Allow Wren to start chats as the 20-point title, explain the browser step in 13-point text with 18-point line spacing, and place the technical scope in a separate 12-point detail row.

13. **Long utility lists lack explicit subgroup headings.** (low; both). Add 12-point Accounts and Presence labels to the account menu and Navigation, Messaging, and Application labels to shortcuts. Use 12-point space before each group and keep command rows at least 28 points tall.

## Screen scores

- chats: **8**
- chats-wide: **7.5**
- chats-narrow: **7**
- chat-no-selection: **8.5**
- chat-filter-no-match: **8.5**
- group-chat: **8**
- channel: **8**
- channel-narrow: **7**
- teams: **8.5**
- calendar: **8**
- calls: **8**
- calls-narrow: **7**
- composer-mention: **8**
- composer-long-draft: **7.5**
- composer-images: **7.5**
- image-preview: **8**
- search-switcher: **8.5**
- new-chat: **7.5**
- settings: **7**
- shortcuts: **8.5**
- status-message: **8**
- account-menu: **8**
- permission-consent: **8**
- error-toast: **8**
- first-sync-loading: **8.5**
- first-sync-empty: **8.5**
- call-bar: **7.5**
- call-docked: **8**
- join-on-phone: **8.5**
- setup: **8**
- signin: **8.5**
- signin-browser: **8.5**
- signin-device-code: **8.5**
- signin-finishing: **8.5**
- signin-expired: **8.5**
- signin-failed: **8.5**

Detailed per-screen issues, original-pixel evidence, and affected screen lists are in B.json. The overall score is holistic, not the arithmetic mean of screen scores.
