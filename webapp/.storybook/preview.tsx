import type { Preview } from '@storybook/react';
import i18n from '../src/i18n';
import { I18nextProvider } from 'react-i18next';
import React from 'react';


const withI18next = (Story: any) => {
  return (
      <I18nextProvider i18n={i18n}>
        <Story />
      </I18nextProvider>
  );
};

const preview: Preview = {
  parameters: {
    actions: { argTypesRegex: '^on[A-Z].*' },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
};


export const decorators = [withI18next];
export default preview;