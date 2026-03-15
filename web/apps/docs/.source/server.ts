// @ts-nocheck
import * as __fd_glob_7 from "../content/skills.mdx?collection=docs"
import * as __fd_glob_6 from "../content/sessions.mdx?collection=docs"
import * as __fd_glob_5 from "../content/quick-start.mdx?collection=docs"
import * as __fd_glob_4 from "../content/manifest-file.mdx?collection=docs"
import * as __fd_glob_3 from "../content/installation.mdx?collection=docs"
import * as __fd_glob_2 from "../content/index.mdx?collection=docs"
import * as __fd_glob_1 from "../content/commands.mdx?collection=docs"
import { default as __fd_glob_0 } from "../content/meta.json?collection=docs"
import { server } from 'fumadocs-mdx/runtime/server';
import type * as Config from '../source.config';

const create = server<typeof Config, import("fumadocs-mdx/runtime/types").InternalTypeConfig & {
  DocData: {
  }
}>({"doc":{"passthroughs":["extractedReferences"]}});

export const docs = await create.docs("docs", "content", {"meta.json": __fd_glob_0, }, {"commands.mdx": __fd_glob_1, "index.mdx": __fd_glob_2, "installation.mdx": __fd_glob_3, "manifest-file.mdx": __fd_glob_4, "quick-start.mdx": __fd_glob_5, "sessions.mdx": __fd_glob_6, "skills.mdx": __fd_glob_7, });