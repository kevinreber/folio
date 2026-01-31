import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    'getting-started',
    'cli-reference',
    {
      type: 'category',
      label: 'Examples',
      items: [
        'examples/index',
        'examples/daily-workflow',
        'examples/interview-prep',
        'examples/performance-reviews',
      ],
    },
    'architecture',
  ],
};

export default sidebars;
