# Wren UI review, round 10 (reviewer A)

**Overall: 8.0 / 10**

| Dimension | Score |
| --- | --- |
| Contrast | 9 |
| Typography | 8 |
| Colour | 8 |
| Spacing | 7.5 |
| Hierarchy | 8 |

## Summary

A calm, coherent and well-built client. Contrast is a real strength: every text sample I measured passes WCAG AA in both themes (body 17.0:1 light / 14.7:1 dark, secondary grey #62656d 5.4-5.8:1, dark #9c9ea8 5.7-6.7:1, accent-on-white buttons 6.1:1, dark orange buttons with near-black text 7.2:1, input borders 3.7-4.8:1). The type scale is disciplined and light/dark are close mirrors. What holds it below 9 is mostly layout and consistency at the edges.

## What blocks a 9

1. Narrow windows aren't adapted: the fixed 232 px list column causes heavy truncation, the hover toolbar is clipped under the header (channel-narrow), and the per-person call buttons disappear (calls-narrow).
2. Light mode: the quote fill (#eeeef0) is the same as the hovered-row fill (#eeeef0), so quotes vanish on hover.
3. Rhythm varies within one flow: sign-in card leading is 17, 20 or 23 px depending on the screen, and button side padding is 9-16 px.
4. Dead space and placement: there are 120-215 px empty bands above short conversations, and dialogs sit anywhere from y=25 to y=120.
5. Stray chrome: a detached composer scrollbar thumb looks like a caret, and settings shows a heavy thumb and a half-clipped checkbox.
6. The light accent #9c4b0c is muddy and close in hue to the away/BRB amber #8a5a00, while the dark accent is a clearly brighter orange.
7. Logic visible in the UI: the calendar is sorted wrong (14:00 below 14:21), and in error-toast the list preview doesn't match the failed message.

## Screen scores

| Screen | Score | Screen | Score |
| --- | --- | --- | --- |
| chats | 8.5 | composer-images | 8.5 |
| chats-wide | 8 | image-preview | 7.5 |
| chats-narrow | 7.5 | search-switcher | 8.5 |
| chat-no-selection | 8.5 | new-chat | 8.5 |
| chat-filter-no-match | 8.5 | settings | 7.5 |
| group-chat | 8.5 | shortcuts | 8 |
| channel | 8 | status-message | 8.5 |
| channel-narrow | 7 | account-menu | 8.5 |
| teams | 8.5 | permission-consent | 8.5 |
| calendar | 7.5 | error-toast | 8 |
| calls | 8 | first-sync-loading | 8.5 |
| calls-narrow | 7 | first-sync-empty | 8 |
| composer-mention | 8 | call-bar | 7.5 |
| composer-long-draft | 7.5 | call-docked | 8 |
| join-on-phone | 7.5 | setup | 8 |
| signin | 8.5 | signin-browser | 8 |
| signin-device-code | 8.5 | signin-finishing | 8 |
| signin-expired | 8.5 | signin-failed | 8.5 |

See grade.json for per-screen issues and the 18 prioritised fixes with measurements.
