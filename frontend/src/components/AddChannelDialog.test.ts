import { mount } from '@vue/test-utils';
import { describe, expect, it } from 'vitest';
import AddChannelDialog from '@/components/AddChannelDialog.vue';
import { testI18n } from '@/test/i18n';

// AppDialog portals its content to document.body via radix-vue DialogPortal,
// which lands outside the test wrapper. Stub it with a passthrough so the
// dialog's buttons stay reachable inside the wrapper element.
const AppDialogStub = {
	name: 'AppDialog',
	props: ['open', 'title'],
	template: '<div data-test="dialog"><slot /></div>'
};

function mountDialog(saving: boolean) {
	return mount(AddChannelDialog, {
		props: { open: true, saving },
		global: {
			plugins: [testI18n],
			stubs: { AppDialog: AppDialogStub }
		}
	});
}

function findCancelButton(wrapper: ReturnType<typeof mountDialog>) {
	const found = wrapper.findAll('button').find((b) => b.text().includes('Cancel'));
	if (!found) throw new Error('Cancel button not found');
	return found;
}

describe('AddChannelDialog', () => {
	it('disables the primary and cancel buttons while saving', () => {
		const wrapper = mountDialog(true);
		const submit = wrapper.find('button[type="submit"]');
		expect((submit.element as HTMLButtonElement).disabled).toBe(true);
		expect((findCancelButton(wrapper).element as HTMLButtonElement).disabled).toBe(true);
	});

	it('shows a spinner and hides the label in the primary button while saving', () => {
		const wrapper = mountDialog(true);
		const submit = wrapper.find('button[type="submit"]');
		expect(submit.find('svg').exists()).toBe(true);
		expect(submit.text()).not.toContain('Create channel');
	});

	it('re-enables the buttons and restores the label when not saving', () => {
		const wrapper = mountDialog(false);
		const submit = wrapper.find('button[type="submit"]');
		expect((submit.element as HTMLButtonElement).disabled).toBe(false);
		expect((findCancelButton(wrapper).element as HTMLButtonElement).disabled).toBe(false);
		expect(submit.find('svg').exists()).toBe(false);
		expect(submit.text()).toContain('Create channel');
	});
});
