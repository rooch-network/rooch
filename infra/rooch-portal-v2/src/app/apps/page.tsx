import type { Project } from 'src/sections/apps/view';

import AppsView from 'src/sections/apps/view';

import { getAvatar } from 'src/utils/avatar';

export const metadata = { title: `Apps` };

export default async function Page() {
  let projects: Project[] = [];

  try {
    const projectsResponse = await fetch(
      `https://api.airtable.com/v0/${process.env.AIRTABLE_BASE_ID}/${process.env.AIRTABLE_PROJECT_TABLE_ID}`,
      {
        headers: {
          Authorization: `Bearer ${process.env.AIRTABLE_ACCESS_TOKEN}`,
          'Content-Type': 'application/json',
        },
        next: {
          revalidate: 60 * 5,
        },
      }
    );

    if (projectsResponse.ok) {
      const projectsRawData = await projectsResponse.json();
      projects = (projectsRawData.records || []).reduce((a: Project[], c: any) => {
        if (c.fields?.Show) {
          try {
            const { fields } = c;
            a.push({
              id: c.id,
              slug: fields.Slug,
              name: fields.Name,
              avatar: getAvatar(fields),
              oneLiner: fields['One-Liner'],
              tags: fields.Tags || [],
            });
          } catch (e) {
            console.log('Error processing project:', e);
          }
        }
        return a;
      }, []);
    } else {
      console.log('Failed to fetch projects:', projectsResponse.status);
    }
  } catch (e) {
    console.log('Error fetching projects from Airtable:', e);
  }

  return <AppsView projects={projects} />;
}
