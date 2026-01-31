import type {ReactNode} from 'react';
import clsx from 'clsx';
import Heading from '@theme/Heading';
import styles from './styles.module.css';

type FeatureItem = {
  title: string;
  description: ReactNode;
  icon: string;
};

const FeatureList: FeatureItem[] = [
  {
    title: 'Capture While Fresh',
    icon: '📝',
    description: (
      <>
        Capture accomplishments as they happen with a simple CLI command.
        Add impact, project context, and importance levels to build a rich
        career history.
      </>
    ),
  },
  {
    title: 'Local-First & Portable',
    icon: '💾',
    description: (
      <>
        Your data stays on your machine in a simple SQLite database.
        Take your career history with you across jobs — it belongs to you,
        not your employer.
      </>
    ),
  },
  {
    title: 'Interview Ready',
    icon: '🎯',
    description: (
      <>
        Turn raw accomplishments into polished resume bullets and STAR stories.
        Never struggle to remember what you did or quantify your impact again.
      </>
    ),
  },
];

function Feature({title, icon, description}: FeatureItem) {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center">
        <span style={{fontSize: '4rem'}}>{icon}</span>
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
