// @ts-nocheck
import { browser } from 'fumadocs-mdx/runtime/browser';
import type * as Config from '../source.config';

const create = browser<typeof Config, import("fumadocs-mdx/runtime/types").InternalTypeConfig & {
  DocData: {
  }
}>();
const browserCollections = {
  docs: create.doc("docs", {"commands.mdx": () => import("../content/commands.mdx?collection=docs"), "index.mdx": () => import("../content/index.mdx?collection=docs"), "installation.mdx": () => import("../content/installation.mdx?collection=docs"), "manifest-file.mdx": () => import("../content/manifest-file.mdx?collection=docs"), "quick-start.mdx": () => import("../content/quick-start.mdx?collection=docs"), "sessions.mdx": () => import("../content/sessions.mdx?collection=docs"), "skills.mdx": () => import("../content/skills.mdx?collection=docs"), }),
};
export default browserCollections;