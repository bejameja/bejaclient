import { createRouter, createWebHashHistory } from 'vue-router'
import HomePage        from '../pages/HomePage.vue'
import ModsPage        from '../pages/ModsPage.vue'
import FriendsPage     from '../pages/FriendsPage.vue'
import QuestsPage      from '../pages/QuestsPage.vue'
import LockerPage      from '../pages/LockerPage.vue'
import ConsolePage     from '../pages/ConsolePage.vue'
import StorePage       from '../pages/StorePage.vue'
import CapesPage       from '../pages/CapesPage.vue'
import ProfilesSettings from '../pages/settings/ProfilesSettings.vue'
import VersionsPage    from '../pages/VersionsPage.vue'
import SettingsPage    from '../pages/SettingsPage.vue'
import LobbyPage       from '../pages/LobbyPage.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/console',   component: ConsolePage,       name: 'console'   },
    { path: '/',          component: HomePage,          name: 'home'      },
    { path: '/cosmetics', component: LockerPage,        name: 'cosmetics' },
    { path: '/store',     component: StorePage,         name: 'store'     },
    { path: '/quests',    component: QuestsPage,        name: 'quests'    },
    { path: '/capes',     component: CapesPage,         name: 'capes'     },
    { path: '/profiles',  component: ProfilesSettings,  name: 'profiles'  },
    { path: '/versions',  component: VersionsPage,      name: 'versions'  },
    { path: '/mods',      component: ModsPage,          name: 'mods'      },
    { path: '/settings',  component: SettingsPage,      name: 'settings'  },
    { path: '/friends',   component: FriendsPage,       name: 'friends'   },
    { path: '/lobby',     component: LobbyPage,         name: 'lobby'     },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
})
